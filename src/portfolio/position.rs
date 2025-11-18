use crate::instrument::FuturesContract;
use serde::{Deserialize, Serialize};

//represents a position in a futures contract
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    //contract symbol
    pub symbol: String,

    //net quantity (positive for long, negative for short, 0 for flat)
    pub net_qty: i32,

    //average entry price
    pub avg_entry_price: f64,

    //realized pnl from closed trades
    pub realized_pnl: f64,
}

impl Position {
    //creates a new flat position
    pub fn new(symbol: String) -> Self {
        Position {
            symbol,
            net_qty: 0,
            avg_entry_price: 0.0,
            realized_pnl: 0.0,
        }
    }

    //calculates unrealized pnl at a given price
    pub fn unrealized_pnl(&self, current_price: f64, contract: &FuturesContract) -> f64 {
        if self.net_qty == 0 {
            return 0.0;
        }

        let price_diff = current_price - self.avg_entry_price;
        contract.pnl_from_price_move(price_diff, self.net_qty)
    }

    //returns true if the position is flat (no open position)
    pub fn is_flat(&self) -> bool {
        self.net_qty == 0
    }

    //returns true if the position is long
    pub fn is_long(&self) -> bool {
        self.net_qty > 0
    }

    //returns true if the position is short
    pub fn is_short(&self) -> bool {
        self.net_qty < 0
    }

    //updates position with a new fill
    //returns the realized pnl from this fill (if it closes/reduces position)
    pub fn update_with_fill(
        &mut self,
        fill_qty: i32,
        fill_price: f64,
        contract: &FuturesContract,
    ) -> f64 {
        let mut realized_pnl = 0.0;

        //if position is flat, just establish new position
        if self.net_qty == 0 {
            self.net_qty = fill_qty;
            self.avg_entry_price = fill_price;
            return realized_pnl;
        }

        //check if this fill is in the same direction or opposite
        let same_direction =
            (self.net_qty > 0 && fill_qty > 0) || (self.net_qty < 0 && fill_qty < 0);

        if same_direction {
            //adding to position - update average entry price
            let total_qty = self.net_qty + fill_qty;
            let total_cost =
                self.avg_entry_price * self.net_qty as f64 + fill_price * fill_qty as f64;
            self.avg_entry_price = total_cost / total_qty as f64;
            self.net_qty = total_qty;
        } else {
            //reducing or reversing position
            let close_qty = fill_qty.abs().min(self.net_qty.abs());

            //calculate realized pnl for the closed portion
            let price_diff = if self.net_qty > 0 {
                //closing long
                fill_price - self.avg_entry_price
            } else {
                //closing short
                self.avg_entry_price - fill_price
            };

            realized_pnl = contract.pnl_from_price_move(price_diff, close_qty);
            self.realized_pnl += realized_pnl;

            //update net quantity
            self.net_qty += fill_qty;

            //if we've reversed the position, set new entry price
            if (self.net_qty > 0 && fill_qty > 0) || (self.net_qty < 0 && fill_qty < 0) {
                self.avg_entry_price = fill_price;
            }

            //if flat, reset entry price
            if self.net_qty == 0 {
                self.avg_entry_price = 0.0;
            }
        }

        realized_pnl
    }

    //returns the notional value of the position
    pub fn notional_value(&self, current_price: f64, contract: &FuturesContract) -> f64 {
        contract.notional_value(current_price, self.net_qty)
    }
}
