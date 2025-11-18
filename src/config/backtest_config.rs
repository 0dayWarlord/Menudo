use crate::instrument::FuturesContract;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

//strategy type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StrategyType {
    SmaCrossover,
    RsiReversion,
}

impl StrategyType {
    //parse strategy type from string
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sma" | "sma_crossover" => Some(StrategyType::SmaCrossover),
            "rsi" | "rsi_reversion" => Some(StrategyType::RsiReversion),
            _ => None,
        }
    }
}

//sma crossover strategy parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmaParams {
    pub fast_window: usize,
    pub slow_window: usize,
    pub qty: u32,
}

impl Default for SmaParams {
    fn default() -> Self {
        SmaParams {
            fast_window: 20,
            slow_window: 50,
            qty: 1,
        }
    }
}

//rsi reversion strategy parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RsiParams {
    pub lookback: usize,
    pub oversold: f64,
    pub overbought: f64,
    pub qty: u32,
}

impl Default for RsiParams {
    fn default() -> Self {
        RsiParams {
            lookback: 14,
            oversold: 30.0,
            overbought: 70.0,
            qty: 1,
        }
    }
}

//strategy-specific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategyParams {
    Sma(SmaParams),
    Rsi(RsiParams),
}

//complete backtest configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfiguration {
    //data
    pub data_path: PathBuf,
    pub symbol: String,

    //contract specification
    pub contract: ContractConfig,

    //account settings
    pub initial_balance: f64,
    pub commission_per_contract: f64,
    pub slippage_per_contract: f64,

    //strategy
    pub strategy_type: StrategyType,
    pub strategy_params: StrategyParams,

    //optional output paths
    pub output_equity_csv: Option<PathBuf>,
    pub output_trades_csv: Option<PathBuf>,
}

//contract configuration (simpler than full futurescontract)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractConfig {
    pub symbol: String,
    pub contract_month: String,
    pub tick_size: f64,
    pub tick_value: f64,
    pub point_value: Option<f64>,
    pub initial_margin: Option<f64>,
    pub maintenance_margin: Option<f64>,
}

impl ContractConfig {
    //converts to a FuturesContract
    pub fn to_futures_contract(&self) -> FuturesContract {
        FuturesContract::from_params(
            self.symbol.clone(),
            self.contract_month.clone(),
            self.tick_size,
            self.tick_value,
            self.point_value,
            self.initial_margin,
            self.maintenance_margin,
        )
    }
}

impl Default for BacktestConfiguration {
    fn default() -> Self {
        BacktestConfiguration {
            data_path: PathBuf::from("data.csv"),
            symbol: "ES".to_string(),
            contract: ContractConfig {
                symbol: "ES".to_string(),
                contract_month: "2025-03".to_string(),
                tick_size: 0.25,
                tick_value: 12.5,
                point_value: Some(50.0),
                initial_margin: Some(13000.0),
                maintenance_margin: Some(12000.0),
            },
            initial_balance: 100000.0,
            commission_per_contract: 2.5,
            slippage_per_contract: 1.0,
            strategy_type: StrategyType::SmaCrossover,
            strategy_params: StrategyParams::Sma(SmaParams::default()),
            output_equity_csv: None,
            output_trades_csv: None,
        }
    }
}

impl BacktestConfiguration {
    //load configuration from a JSON file
    pub fn from_json_file(path: &PathBuf) -> anyhow::Result<Self> {
        let contents = std::fs::read_to_string(path)?;
        let config: BacktestConfiguration = serde_json::from_str(&contents)?;
        Ok(config)
    }

    //save configuration to a JSON file
    pub fn to_json_file(&self, path: &PathBuf) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        std::fs::write(path, json)?;
        Ok(()        )
    }
}
