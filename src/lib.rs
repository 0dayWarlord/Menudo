//a Rust-based strategy backtesting engine for futures contracts

pub mod config;
pub mod data;
pub mod engine;
pub mod instrument;
pub mod metrics;
pub mod portfolio;
pub mod strategy;

//prelude module for convenient imports
pub mod prelude {
    pub use crate::config::{
        BacktestConfiguration, ContractConfig, RsiParams, SmaParams, StrategyParams, StrategyType,
    };
    pub use crate::data::{filter_by_symbol, load_csv, Bar};
    pub use crate::engine::{
        BacktestConfig, BacktestEngine, BacktestResult, ExecutionEngine, Fill, Order, OrderSide,
        OrderType,
    };
    pub use crate::instrument::FuturesContract;
    pub use crate::metrics::{calculate_equity_curve, EquityPoint, SummaryMetrics};
    pub use crate::portfolio::{Account, Position};
    pub use crate::strategy::{
        rsi_reversion::RsiReversionStrategy, sma_crossover::SmaCrossoverStrategy, Strategy,
        StrategyContext,
    };
}
