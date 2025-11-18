use serde::{Deserialize, Serialize};

//represents a futures contract specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesContract {
    //contract symbol (eg es, nq, cl)
    pub symbol: String,

    //contract month (eg 2025-03, h25)
    pub contract_month: String,

    //minimum price fluctuation
    pub tick_size: f64,

    //dollar value of one tick
    pub tick_value: f64,

    //dollar value of one full point move
    pub point_value: f64,

    //exchange where traded
    pub exchange: String,

    //currency denomination
    pub currency: String,

    //contract multiplier (often same as point_value)
    pub multiplier: f64,

    //initial margin per contract
    pub initial_margin: f64,

    //maintenance margin per contract
    pub maintenance_margin: f64,
}

impl FuturesContract {
    //creates a new futurescontract
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        symbol: String,
        contract_month: String,
        tick_size: f64,
        tick_value: f64,
        point_value: f64,
        exchange: String,
        currency: String,
        multiplier: f64,
        initial_margin: f64,
        maintenance_margin: f64,
    ) -> Self {
        FuturesContract {
            symbol,
            contract_month,
            tick_size,
            tick_value,
            point_value,
            exchange,
            currency,
            multiplier,
            initial_margin,
            maintenance_margin,
        }
    }

    //converts a price difference to ticks
    pub fn price_to_ticks(&self, price_diff: f64) -> f64 {
        price_diff / self.tick_size
    }

    //calculates pnl from a price move
    //arguments
    //price_diff - the price difference (exit_price - entry_price for long)
    //quantity - number of contracts (positive for long, negative for short)
    pub fn pnl_from_price_move(&self, price_diff: f64, quantity: i32) -> f64 {
        let ticks = self.price_to_ticks(price_diff);
        ticks * self.tick_value * quantity as f64
    }

    //calculates the notional value of a position
    pub fn notional_value(&self, price: f64, quantity: i32) -> f64 {
        price * self.multiplier * quantity.abs() as f64
    }

    //returns the initial margin requirement for a given quantity
    pub fn initial_margin_requirement(&self, quantity: i32) -> f64 {
        self.initial_margin * quantity.abs() as f64
    }

    //returns the maintenance margin requirement for a given quantity
    pub fn maintenance_margin_requirement(&self, quantity: i32) -> f64 {
        self.maintenance_margin * quantity.abs() as f64
    }

    //helper to create an e-mini s&p 500 contract
    pub fn es(contract_month: &str) -> Self {
        FuturesContract::new(
            "ES".to_string(),
            contract_month.to_string(),
            0.25,  //tick_size
            12.50, //tick_value (0.25 * 50)
            50.0,  //point_value
            "CME".to_string(),
            "USD".to_string(),
            50.0,    //multiplier
            13000.0, //initial_margin (approximate)
            12000.0, //maintenance_margin (approximate)
        )
    }

    //helper to create an e-mini nasdaq-100 contract
    pub fn nq(contract_month: &str) -> Self {
        FuturesContract::new(
            "NQ".to_string(),
            contract_month.to_string(),
            0.25, //tick_size
            5.0,  //tick_value (0.25 * 20)
            20.0, //point_value
            "CME".to_string(),
            "USD".to_string(),
            20.0,    //multiplier
            17000.0, //initial_margin (approximate)
            15500.0, //maintenance_margin (approximate)
        )
    }

    //helper to create a custom contract from cli parameters
    pub fn from_params(
        symbol: String,
        contract_month: String,
        tick_size: f64,
        tick_value: f64,
        point_value: Option<f64>,
        initial_margin: Option<f64>,
        maintenance_margin: Option<f64>,
    ) -> Self {
        let point_value = point_value.unwrap_or(tick_value / tick_size);
        let multiplier = point_value;
        let initial_margin = initial_margin.unwrap_or(10000.0);
        let maintenance_margin = maintenance_margin.unwrap_or(initial_margin * 0.8);

        FuturesContract::new(
            symbol,
            contract_month,
            tick_size,
            tick_value,
            point_value,
            "UNKNOWN".to_string(),
            "USD".to_string(),
            multiplier,
            initial_margin,
            maintenance_margin,
        )
    }
}
