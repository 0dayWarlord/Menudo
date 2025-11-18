use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

//a point in the equity curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    pub timestamp: DateTime<Utc>,
    pub equity: f64,
    pub drawdown: f64,
    pub returns: f64,
}

impl EquityPoint {
    pub fn new(timestamp: DateTime<Utc>, equity: f64, drawdown: f64, returns: f64) -> Self {
        EquityPoint {
            timestamp,
            equity,
            drawdown,
            returns,
        }
    }
}

//calculates the equity curve with drawdowns
pub fn calculate_equity_curve(
    timestamps: &[DateTime<Utc>],
    equity_values: &[f64],
    initial_balance: f64,
) -> Vec<EquityPoint> {
    let mut curve = Vec::with_capacity(timestamps.len());
    let mut peak = initial_balance;
    let mut prev_equity = initial_balance;

    for (i, (&timestamp, &equity)) in timestamps.iter().zip(equity_values.iter()).enumerate() {
        //update peak
        if equity > peak {
            peak = equity;
        }

        //calculate drawdown
        let drawdown = if peak > 0.0 {
            (peak - equity) / peak
        } else {
            0.0
        };

        //calculate returns
        let returns = if i == 0 {
            0.0
        } else {
            (equity - prev_equity) / prev_equity
        };

        curve.push(EquityPoint::new(timestamp, equity, drawdown, returns));
        prev_equity = equity;
    }

    curve
}

//calculates maximum drawdown from equity curve
pub fn max_drawdown(equity_curve: &[EquityPoint]) -> f64 {
    equity_curve
        .iter()
        .map(|point| point.drawdown)
        .fold(0.0, f64::max)
}

//calculates returns from equity values
pub fn calculate_returns(equity_values: &[f64]) -> Vec<f64> {
    if equity_values.len() < 2 {
        return vec![];
    }

    let mut returns = Vec::with_capacity(equity_values.len() - 1);
    for i in 1..equity_values.len() {
        let ret = (equity_values[i] - equity_values[i - 1]) / equity_values[i - 1];
        returns.push(ret);
    }
    returns
}
