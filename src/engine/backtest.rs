use crate::data::Bar;
use crate::engine::execution::ExecutionEngine;
use crate::instrument::FuturesContract;
use crate::metrics::{calculate_equity_curve, EquityPoint, SummaryMetrics};
use crate::portfolio::Account;
use crate::strategy::{Strategy, StrategyContext};
use std::collections::HashMap;

//result of a backtest
#[derive(Debug, Clone)]
pub struct BacktestResult {
    pub summary: SummaryMetrics,
    pub equity_curve: Vec<EquityPoint>,
    pub trades: Vec<crate::engine::execution::Fill>,
}

//configuration for a backtest
#[derive(Debug, Clone)]
pub struct BacktestConfig {
    pub initial_balance: f64,
    pub commission_per_contract: f64,
    pub slippage_per_contract: f64,
    pub max_lookback: usize,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        BacktestConfig {
            initial_balance: 100000.0,
            commission_per_contract: 2.5,
            slippage_per_contract: 1.0,
            max_lookback: 500,
        }
    }
}

//main backtest engine
pub struct BacktestEngine {
    config: BacktestConfig,
    bars: Vec<Bar>,
    contract: FuturesContract,
    account: Account,
    execution: ExecutionEngine,
    equity_history: Vec<(chrono::DateTime<chrono::Utc>, f64)>,
}

impl BacktestEngine {
    //creates a new backtest engine
    pub fn new(config: BacktestConfig, bars: Vec<Bar>, contract: FuturesContract) -> Self {
        let account = Account::new(
            config.initial_balance,
            config.commission_per_contract,
            config.slippage_per_contract,
        );

        BacktestEngine {
            config,
            bars,
            contract,
            account,
            execution: ExecutionEngine::new(),
            equity_history: Vec::new(),
        }
    }

    //runs the backtest with the given strategy
    pub fn run(&mut self, strategy: &mut Box<dyn Strategy>) -> BacktestResult {
        //create strategy context
        let mut context = StrategyContext::new(
            self.contract.symbol.clone(),
            self.config.max_lookback,
            &mut self.execution as *mut ExecutionEngine,
            &mut self.account as *mut Account,
        );

        //call strategy initialization
        strategy.on_start(&mut context);

        //main backtest loop
        for i in 0..self.bars.len() {
            let bar = self.bars[i].clone();

            //update context with new bar
            context.push_bar(bar.clone());

            //call strategy
            strategy.on_bar(&mut context, &bar);

            //process any pending orders from previous bar
            //orders submitted on this bar will be filled at next bar's open
            if i > 0 {
                let fills = self.execution.process_orders(bar.open, bar.high, bar.low);

                //process fills
                for fill in fills {
                    self.account.process_fill(fill, &self.contract);
                }
            }

            //update account equity
            let mut prices = HashMap::new();
            prices.insert(self.contract.symbol.clone(), bar.close);

            let mut contracts = HashMap::new();
            contracts.insert(self.contract.symbol.clone(), self.contract.clone());

            self.account.update_equity(&prices, &contracts);

            //record equity
            self.equity_history
                .push((bar.timestamp, self.account.equity));
        }

        //process any remaining orders at final bar
        if !self.bars.is_empty() {
            let last_bar = self.bars.last().unwrap();
            let fills = self
                .execution
                .process_orders(last_bar.close, last_bar.high, last_bar.low);

            for fill in fills {
                self.account.process_fill(fill, &self.contract);
            }
        }

        //call strategy finalization
        strategy.on_end(&mut context);

        //process final orders
        if !self.bars.is_empty() {
            let last_bar = self.bars.last().unwrap();
            let fills = self
                .execution
                .process_orders(last_bar.close, last_bar.high, last_bar.low);

            for fill in fills {
                self.account.process_fill(fill, &self.contract);
            }

            //final equity update
            let mut prices = HashMap::new();
            prices.insert(self.contract.symbol.clone(), last_bar.close);

            let mut contracts = HashMap::new();
            contracts.insert(self.contract.symbol.clone(), self.contract.clone());

            self.account.update_equity(&prices, &contracts);

            //update final equity in history
            if let Some(last) = self.equity_history.last_mut() {
                last.1 = self.account.equity;
            }
        }

        //build result
        self.build_result()
    }

    fn build_result(&self) -> BacktestResult {
        let timestamps: Vec<_> = self.equity_history.iter().map(|(t, _)| *t).collect();
        let equity_values: Vec<_> = self.equity_history.iter().map(|(_, e)| *e).collect();

        let equity_curve =
            calculate_equity_curve(&timestamps, &equity_values, self.config.initial_balance);

        let trades = self.account.trade_log.clone();

        let summary =
            SummaryMetrics::from_backtest(&equity_curve, &trades, self.config.initial_balance);

        BacktestResult {
            summary,
            equity_curve,
            trades,
        }
    }

    //returns a reference to the account
    pub fn account(&self) -> &Account {
        &self.account
    }

    //returns a reference to the contract
    pub fn contract(&self) -> &FuturesContract {
        &self.contract
    }
}
