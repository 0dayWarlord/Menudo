use crate::engine::execution::Fill;
use crate::instrument::FuturesContract;
use crate::portfolio::position::Position;
use std::collections::HashMap;

//represents a trading account with positions and cash
#[derive(Debug, Clone)]
pub struct Account {
    //initial account balance
    pub initial_balance: f64,

    //current cash (includes realized pnl, subtracts commissions)
    pub cash: f64,

    //current total equity (cash + unrealized pnl)
    pub equity: f64,

    //total margin currently in use
    pub margin_used: f64,

    //open positions by symbol
    pub open_positions: HashMap<String, Position>,

    //complete trade log
    pub trade_log: Vec<Fill>,

    //commission per contract per side
    pub commission_per_contract: f64,

    //slippage per contract per side
    pub slippage_per_contract: f64,
}

impl Account {
    //creates a new account with initial balance
    pub fn new(
        initial_balance: f64,
        commission_per_contract: f64,
        slippage_per_contract: f64,
    ) -> Self {
        Account {
            initial_balance,
            cash: initial_balance,
            equity: initial_balance,
            margin_used: 0.0,
            open_positions: HashMap::new(),
            trade_log: Vec::new(),
            commission_per_contract,
            slippage_per_contract,
        }
    }

    //processes a fill and updates the account
    pub fn process_fill(&mut self, fill: Fill, contract: &FuturesContract) {
        //calculate total costs (commission + slippage)
        let total_cost =
            (self.commission_per_contract + self.slippage_per_contract) * fill.qty.abs() as f64;

        //deduct costs from cash
        self.cash -= total_cost;

        //get or create position
        let position = self
            .open_positions
            .entry(fill.symbol.clone())
            .or_insert_with(|| Position::new(fill.symbol.clone()));

        //update position and get realized pnl
        let realized_pnl = position.update_with_fill(fill.qty, fill.fill_price, contract);

        //add realized pnl to cash
        self.cash += realized_pnl;

        //update margin used
        self.update_margin_used(contract);

        //log the fill
        self.trade_log.push(fill);
    }

    //updates total equity based on current market prices
    pub fn update_equity(
        &mut self,
        prices: &HashMap<String, f64>,
        contracts: &HashMap<String, FuturesContract>,
    ) {
        let mut total_unrealized_pnl = 0.0;

        for (symbol, position) in &self.open_positions {
            if let (Some(&price), Some(contract)) = (prices.get(symbol), contracts.get(symbol)) {
                total_unrealized_pnl += position.unrealized_pnl(price, contract);
            }
        }

        self.equity = self.cash + total_unrealized_pnl;
    }

    //updates margin used based on current positions
    fn update_margin_used(&mut self, contract: &FuturesContract) {
        self.margin_used = 0.0;

        for position in self.open_positions.values() {
            if !position.is_flat() {
                self.margin_used += contract.initial_margin_requirement(position.net_qty);
            }
        }
    }

    //returns the position for a symbol, or none if flat
    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.open_positions.get(symbol)
    }

    //returns available buying power (cash - margin_used)
    pub fn buying_power(&self) -> f64 {
        self.cash - self.margin_used
    }

    //checks if the account has sufficient margin for a new position
    pub fn has_sufficient_margin(&self, required_margin: f64) -> bool {
        self.buying_power() >= required_margin
    }

    //checks for margin breach (equity below maintenance margin)
    pub fn is_margin_breach(&self, contracts: &HashMap<String, FuturesContract>) -> bool {
        let mut total_maintenance_margin = 0.0;

        for (symbol, position) in &self.open_positions {
            if let Some(contract) = contracts.get(symbol) {
                if !position.is_flat() {
                    total_maintenance_margin +=
                        contract.maintenance_margin_requirement(position.net_qty);
                }
            }
        }

        self.equity < total_maintenance_margin
    }

    //returns total realized pnl across all positions
    pub fn total_realized_pnl(&self) -> f64 {
        self.open_positions.values().map(|p| p.realized_pnl).sum()
    }

    //returns total unrealized pnl at given prices
    pub fn total_unrealized_pnl(
        &self,
        prices: &HashMap<String, f64>,
        contracts: &HashMap<String, FuturesContract>,
    ) -> f64 {
        let mut total = 0.0;

        for (symbol, position) in &self.open_positions {
            if let (Some(&price), Some(contract)) = (prices.get(symbol), contracts.get(symbol)) {
                total += position.unrealized_pnl(price, contract);
            }
        }

        total
    }

    //returns total pnl (realized + unrealized)
    pub fn total_pnl(
        &self,
        prices: &HashMap<String, f64>,
        contracts: &HashMap<String, FuturesContract>,
    ) -> f64 {
        self.total_realized_pnl() + self.total_unrealized_pnl(prices, contracts)
    }

    //returns the total return as a percentage
    pub fn total_return(&self) -> f64 {
        (self.equity - self.initial_balance) / self.initial_balance
    }
}
