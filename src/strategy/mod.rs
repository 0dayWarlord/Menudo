pub mod rsi_reversion;
pub mod sma_crossover;

use crate::data::Bar;
use crate::engine::execution::{ExecutionEngine, OrderSide};
use crate::portfolio::{Account, Position};
use chrono::{DateTime, Utc};
use std::collections::VecDeque;

//strategy interface that all strategies must implement
pub trait Strategy: Send {
    //called once at the start of the backtest
    fn on_start(&mut self, context: &mut StrategyContext);

    //called on each new bar
    fn on_bar(&mut self, context: &mut StrategyContext, bar: &Bar);

    //called at the end of the backtest
    fn on_end(&mut self, context: &mut StrategyContext);

    //returns the strategy name
    fn name(&self) -> &str;
}

//context providing access to market data and order submission
pub struct StrategyContext {
    //symbol being traded
    pub symbol: String,

    //historical bars (ring buffer with limited lookback)
    bar_history: VecDeque<Bar>,

    //maximum bars to keep in history
    max_history: usize,

    //current timestamp
    pub current_time: DateTime<Utc>,

    //reference to execution engine
    execution_engine: *mut ExecutionEngine,

    //reference to account
    account: *mut Account,
}

impl StrategyContext {
    //creates a new strategy context
    pub fn new(
        symbol: String,
        max_history: usize,
        execution_engine: *mut ExecutionEngine,
        account: *mut Account,
    ) -> Self {
        StrategyContext {
            symbol,
            bar_history: VecDeque::with_capacity(max_history),
            max_history,
            current_time: Utc::now(),
            execution_engine,
            account,
        }
    }

    //adds a bar to the history
    pub fn push_bar(&mut self, bar: Bar) {
        self.current_time = bar.timestamp;

        if self.bar_history.len() >= self.max_history {
            self.bar_history.pop_front();
        }
        self.bar_history.push_back(bar);
    }

    //returns the last n bars (most recent first)
    pub fn get_bars(&self, n: usize) -> Vec<&Bar> {
        let len = self.bar_history.len();
        let start = len.saturating_sub(n);
        self.bar_history.range(start..).collect()
    }

    //returns all available bars
    pub fn get_all_bars(&self) -> Vec<&Bar> {
        self.bar_history.iter().collect()
    }

    //returns the most recent bar
    pub fn last_bar(&self) -> Option<&Bar> {
        self.bar_history.back()
    }

    //returns the close prices for the last n bars
    pub fn get_close_prices(&self, n: usize) -> Vec<f64> {
        let bars = self.get_bars(n);
        bars.iter().map(|b| b.close).collect()
    }

    //submits a market order
    pub fn market_order(&mut self, symbol: String, qty: u32, side: OrderSide) -> u64 {
        unsafe { (*self.execution_engine).market_order(self.current_time, symbol, qty, side) }
    }

    //submits a limit order
    pub fn limit_order(
        &mut self,
        symbol: String,
        qty: u32,
        side: OrderSide,
        limit_price: f64,
    ) -> u64 {
        unsafe {
            (*self.execution_engine).limit_order(self.current_time, symbol, qty, side, limit_price)
        }
    }

    //returns the current position for the strategy's symbol
    pub fn current_position(&self) -> Option<&Position> {
        unsafe { (*self.account).get_position(&self.symbol) }
    }

    //returns the current cash balance
    pub fn cash(&self) -> f64 {
        unsafe { (*self.account).cash }
    }

    //returns the current equity
    pub fn equity(&self) -> f64 {
        unsafe { (*self.account).equity }
    }

    //returns the number of bars in history
    pub fn bar_count(&self) -> usize {
        self.bar_history.len()
    }

    //cancels all pending orders
    pub fn cancel_all_orders(&mut self) {
        unsafe {
            (*self.execution_engine).cancel_all_orders();
        }
    }
}

//helper function to calculate simple moving average
pub fn sma(prices: &[f64]) -> Option<f64> {
    if prices.is_empty() {
        return None;
    }
    Some(prices.iter().sum::<f64>() / prices.len() as f64)
}

//helper function to calculate relative strength index
pub fn rsi(prices: &[f64], period: usize) -> Option<f64> {
    if prices.len() < period + 1 {
        return None;
    }

    let mut gains = Vec::new();
    let mut losses = Vec::new();

    for i in 1..prices.len() {
        let change = prices[i] - prices[i - 1];
        if change > 0.0 {
            gains.push(change);
            losses.push(0.0);
        } else {
            gains.push(0.0);
            losses.push(-change);
        }
    }

    if gains.len() < period {
        return None;
    }

    let avg_gain: f64 = gains.iter().rev().take(period).sum::<f64>() / period as f64;
    let avg_loss: f64 = losses.iter().rev().take(period).sum::<f64>() / period as f64;

    if avg_loss == 0.0 {
        return Some(100.0);
    }

    let rs = avg_gain / avg_loss;
    Some(100.0 - (100.0 / (1.0 + rs)))
}
