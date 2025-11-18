use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

//order side (buy or sell)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderSide {
    Buy,
    Sell,
}

impl OrderSide {
    //converts to quantity sign (Buy = +1, Sell = -1)
    pub fn to_qty_sign(&self) -> i32 {
        match self {
            OrderSide::Buy => 1,
            OrderSide::Sell => -1,
        }
    }
}

//order type
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum OrderType {
    Market,
    Limit,
    Stop,
}

//represents a trading order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    pub id: u64,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub qty: u32,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub limit_price: Option<f64>,
    pub stop_price: Option<f64>,
}

impl Order {
    //creates a new market order
    pub fn market(
        id: u64,
        timestamp: DateTime<Utc>,
        symbol: String,
        qty: u32,
        side: OrderSide,
    ) -> Self {
        Order {
            id,
            timestamp,
            symbol,
            qty,
            side,
            order_type: OrderType::Market,
            limit_price: None,
            stop_price: None,
        }
    }

    //creates a new limit order
    pub fn limit(
        id: u64,
        timestamp: DateTime<Utc>,
        symbol: String,
        qty: u32,
        side: OrderSide,
        limit_price: f64,
    ) -> Self {
        Order {
            id,
            timestamp,
            symbol,
            qty,
            side,
            order_type: OrderType::Limit,
            limit_price: Some(limit_price),
            stop_price: None,
        }
    }

    //creates a new stop order
    pub fn stop(
        id: u64,
        timestamp: DateTime<Utc>,
        symbol: String,
        qty: u32,
        side: OrderSide,
        stop_price: f64,
    ) -> Self {
        Order {
            id,
            timestamp,
            symbol,
            qty,
            side,
            order_type: OrderType::Stop,
            limit_price: None,
            stop_price: Some(stop_price),
        }
    }

    //returns the signed quantity (positive for buy, negative for sell)
    pub fn signed_qty(&self) -> i32 {
        (self.qty as i32) * self.side.to_qty_sign()
    }
}

//represents a filled order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    pub id: u64,
    pub order_id: u64,
    pub timestamp: DateTime<Utc>,
    pub symbol: String,
    pub qty: i32, //signed: positive for long, negative for short
    pub side: OrderSide,
    pub fill_price: f64,
    pub fees: f64, //total fees (commission + slippage)
}

impl Fill {
    pub fn from_order(fill_id: u64, order: &Order, fill_price: f64, fees: f64) -> Self {
        Fill {
            id: fill_id,
            order_id: order.id,
            timestamp: order.timestamp,
            symbol: order.symbol.clone(),
            qty: order.signed_qty(),
            side: order.side,
            fill_price,
            fees,
        }
    }

    //returns the notional value of the fill
    pub fn notional_value(&self, multiplier: f64) -> f64 {
        self.fill_price * multiplier * self.qty.abs() as f64
    }
}

//simulates order execution
pub struct ExecutionEngine {
    next_order_id: u64,
    next_fill_id: u64,
    pending_orders: Vec<Order>,
}

impl ExecutionEngine {
    pub fn new() -> Self {
        ExecutionEngine {
            next_order_id: 1,
            next_fill_id: 1,
            pending_orders: Vec::new(),
        }
    }

    //submits an order and returns its ID
    pub fn submit_order(&mut self, order: Order) -> u64 {
        let id = order.id;
        self.pending_orders.push(order);
        id
    }

    //creates and submits a market order
    pub fn market_order(
        &mut self,
        timestamp: DateTime<Utc>,
        symbol: String,
        qty: u32,
        side: OrderSide,
    ) -> u64 {
        let order = Order::market(self.next_order_id, timestamp, symbol, qty, side);
        self.next_order_id += 1;
        self.submit_order(order)
    }

    //creates and submits a limit order
    pub fn limit_order(
        &mut self,
        timestamp: DateTime<Utc>,
        symbol: String,
        qty: u32,
        side: OrderSide,
        limit_price: f64,
    ) -> u64 {
        let order = Order::limit(
            self.next_order_id,
            timestamp,
            symbol,
            qty,
            side,
            limit_price,
        );
        self.next_order_id += 1;
        self.submit_order(order)
    }

    //processes pending orders against current bar and returns fills
    //market orders fill at the open of the next bar
    //limit orders fill if price crosses the limit during the bar
    pub fn process_orders(&mut self, bar_open: f64, bar_high: f64, bar_low: f64) -> Vec<Fill> {
        let mut fills = Vec::new();
        let mut orders_to_keep = Vec::new();

        for order in self.pending_orders.drain(..) {
            match order.order_type {
                OrderType::Market => {
                    //market orders fill at bar open
                    let fill = Fill::from_order(self.next_fill_id, &order, bar_open, 0.0);
                    self.next_fill_id += 1;
                    fills.push(fill);
                }
                OrderType::Limit => {
                    //limit buy fills if low <= limit_price
                    //limit sell fills if high >= limit_price
                    if let Some(limit_price) = order.limit_price {
                        let filled = match order.side {
                            OrderSide::Buy => bar_low <= limit_price,
                            OrderSide::Sell => bar_high >= limit_price,
                        };

                        if filled {
                            let fill =
                                Fill::from_order(self.next_fill_id, &order, limit_price, 0.0);
                            self.next_fill_id += 1;
                            fills.push(fill);
                        } else {
                            //keep for next bar
                            orders_to_keep.push(order);
                        }
                    }
                }
                OrderType::Stop => {
                    //stop buy triggers if high >= stop_price
                    //stop sell triggers if low <= stop_price
                    if let Some(stop_price) = order.stop_price {
                        let triggered = match order.side {
                            OrderSide::Buy => bar_high >= stop_price,
                            OrderSide::Sell => bar_low <= stop_price,
                        };

                        if triggered {
                            //once triggered, fills at stop price
                            let fill =
                                Fill::from_order(self.next_fill_id, &order, stop_price, 0.0);
                            self.next_fill_id += 1;
                            fills.push(fill);
                        } else {
                            //keep for next bar
                            orders_to_keep.push(order);
                        }
                    }
                }
            }
        }

        self.pending_orders = orders_to_keep;
        fills
    }

    //returns the number of pending orders
    pub fn pending_order_count(&self) -> usize {
        self.pending_orders.len()
    }

    //cancels all pending orders
    pub fn cancel_all_orders(&mut self) {
        self.pending_orders.clear();
    }
}

impl Default for ExecutionEngine {
    fn default() -> Self {
        Self::new()
    }
}
