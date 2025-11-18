use crate::data::Bar;
use crate::engine::execution::OrderSide;
use crate::strategy::{sma, Strategy, StrategyContext};

//sma crossover strategy
//goes long when fast sma crosses above slow sma
//goes short when fast sma crosses below slow sma
#[derive(Debug, Clone)]
pub struct SmaCrossoverStrategy {
    symbol: String,
    fast_window: usize,
    slow_window: usize,
    qty: u32,

    //state
    last_fast_sma: Option<f64>,
    last_slow_sma: Option<f64>,
}

impl SmaCrossoverStrategy {
    pub fn new(symbol: String, fast_window: usize, slow_window: usize, qty: u32) -> Self {
        SmaCrossoverStrategy {
            symbol,
            fast_window,
            slow_window,
            qty,
            last_fast_sma: None,
            last_slow_sma: None,
        }
    }

    //checks for crossover and returns signal
    //returns some(orderside buy) for bullish crossover
    //returns some(orderside sell) for bearish crossover
    //returns none for no crossover
    fn check_crossover(&self, fast_sma: f64, slow_sma: f64) -> Option<OrderSide> {
        if let (Some(prev_fast), Some(prev_slow)) = (self.last_fast_sma, self.last_slow_sma) {
            //bullish crossover fast crosses above slow
            if prev_fast <= prev_slow && fast_sma > slow_sma {
                return Some(OrderSide::Buy);
            }
            //bearish crossover fast crosses below slow
            if prev_fast >= prev_slow && fast_sma < slow_sma {
                return Some(OrderSide::Sell);
            }
        }
        None
    }
}

impl Strategy for SmaCrossoverStrategy {
    fn on_start(&mut self, _context: &mut StrategyContext) {
        //initialize state
        self.last_fast_sma = None;
        self.last_slow_sma = None;
    }

    fn on_bar(&mut self, context: &mut StrategyContext, _bar: &Bar) {
        //need at least slow_window bars to calculate
        if context.bar_count() < self.slow_window {
            return;
        }

        //get close prices
        let closes = context.get_close_prices(self.slow_window);

        //calculate smas
        let fast_prices = &closes[closes.len().saturating_sub(self.fast_window)..];
        let slow_prices = &closes;

        let fast_sma = match sma(fast_prices) {
            Some(v) => v,
            None => return,
        };

        let slow_sma = match sma(slow_prices) {
            Some(v) => v,
            None => return,
        };

        //check for crossover
        if let Some(signal) = self.check_crossover(fast_sma, slow_sma) {
            //get current position
            let current_position = context.current_position();
            let current_quantity = current_position.map(|p| p.net_qty).unwrap_or(0);

            match signal {
                OrderSide::Buy => {
                    //go long if flat or short, buy to establish long position
                    if current_quantity <= 0 {
                        let quantity_to_buy = if current_quantity < 0 {
                            //close short and open long
                            (current_quantity.abs() + self.qty as i32) as u32
                        } else {
                            //just open long
                            self.qty
                        };

                        context.market_order(self.symbol.clone(), quantity_to_buy, OrderSide::Buy);
                    }
                }
                OrderSide::Sell => {
                    //go short if flat or long, sell to establish short position
                    if current_quantity >= 0 {
                        let quantity_to_sell = if current_quantity > 0 {
                            //close long and open short
                            (current_quantity.abs() + self.qty as i32) as u32
                        } else {
                            //just open short
                            self.qty
                        };

                        context.market_order(self.symbol.clone(), quantity_to_sell, OrderSide::Sell);
                    }
                }
            }
        }

        //update state
        self.last_fast_sma = Some(fast_sma);
        self.last_slow_sma = Some(slow_sma);
    }

    fn on_end(&mut self, context: &mut StrategyContext) {
        //close any open positions
        if let Some(position) = context.current_position() {
            if !position.is_flat() {
                let quantity = position.net_qty.unsigned_abs();
                let side = if position.net_qty > 0 {
                    OrderSide::Sell
                } else {
                    OrderSide::Buy
                };
                context.market_order(self.symbol.clone(), quantity, side);
            }
        }
    }

    fn name(&self) -> &str {
        "SMA Crossover"
    }
}
