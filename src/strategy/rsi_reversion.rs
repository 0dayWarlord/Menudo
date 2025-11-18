use crate::data::Bar;
use crate::engine::execution::OrderSide;
use crate::strategy::{rsi, Strategy, StrategyContext};

//rsi mean reversion strategy
//buys when rsi drops below oversold threshold
//sells when rsi rises above overbought threshold
#[derive(Debug, Clone)]
pub struct RsiReversionStrategy {
    symbol: String,
    lookback: usize,
    oversold: f64,
    overbought: f64,
    qty: u32,
}

impl RsiReversionStrategy {
    pub fn new(symbol: String, lookback: usize, oversold: f64, overbought: f64, qty: u32) -> Self {
        RsiReversionStrategy {
            symbol,
            lookback,
            oversold,
            overbought,
            qty,
        }
    }

    //default rsi strategy with standard parameters
    pub fn default(symbol: String, qty: u32) -> Self {
        Self::new(symbol, 14, 30.0, 70.0, qty)
    }
}

impl Strategy for RsiReversionStrategy {
    fn on_start(&mut self, _context: &mut StrategyContext) {
        //no initialization needed
    }

    fn on_bar(&mut self, context: &mut StrategyContext, _bar: &Bar) {
        //need at least lookback + 1 bars for rsi calculation
        if context.bar_count() < self.lookback + 1 {
            return;
        }

        //get close prices
        let closes = context.get_close_prices(self.lookback + 1);

        //calculate rsi
        let rsi_value = match rsi(&closes, self.lookback) {
            Some(v) => v,
            None => return,
        };

        //get current position
        let current_position = context.current_position();
        let current_quantity = current_position.map(|p| p.net_qty).unwrap_or(0);

        //trading logic
        if rsi_value < self.oversold {
            //oversold - go long if not already
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
        } else if rsi_value > self.overbought {
            //overbought - go short if not already
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
        } else {
            //in neutral zone - close positions if open
            if current_quantity != 0 {
                let quantity = current_quantity.unsigned_abs();
                let side = if current_quantity > 0 {
                    OrderSide::Sell
                } else {
                    OrderSide::Buy
                };
                context.market_order(self.symbol.clone(), quantity, side);
            }
        }
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
        "RSI Reversion"
    }
}
