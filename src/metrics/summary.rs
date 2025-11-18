use crate::engine::execution::Fill;
use crate::metrics::timeseries::{calculate_returns, max_drawdown, EquityPoint};
use prettytable::{Cell, Row, Table};
use serde::{Deserialize, Serialize};
use statrs::statistics::Statistics;

//summary metrics for a backtest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryMetrics {
    pub initial_balance: f64,
    pub final_balance: f64,
    pub total_return: f64,
    pub total_return_pct: f64,
    pub cagr: f64,
    pub max_drawdown: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub win_rate: f64,
    pub avg_win: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub num_trades: usize,
    pub num_winning_trades: usize,
    pub num_losing_trades: usize,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub exposure: f64,
}

impl SummaryMetrics {
    //calculate summary metrics from equity curve and trade log
    pub fn from_backtest(
        equity_curve: &[EquityPoint],
        trades: &[Fill],
        initial_balance: f64,
    ) -> Self {
        let final_balance = equity_curve
            .last()
            .map(|p| p.equity)
            .unwrap_or(initial_balance);

        let total_return = final_balance - initial_balance;
        let total_return_pct = total_return / initial_balance;

        //calculate cagr
        let cagr = if equity_curve.len() >= 2 {
            let start_time = equity_curve.first().unwrap().timestamp;
            let end_time = equity_curve.last().unwrap().timestamp;
            let duration_days = (end_time - start_time).num_days() as f64;
            let years = duration_days / 365.25;

            if years > 0.0 {
                ((final_balance / initial_balance).powf(1.0 / years) - 1.0) * 100.0
            } else {
                0.0
            }
        } else {
            0.0
        };

        //max drawdown
        let max_dd = max_drawdown(equity_curve);

        //calculate returns for sharpe and sortino
        let equity_values: Vec<f64> = equity_curve.iter().map(|p| p.equity).collect();
        let returns = calculate_returns(&equity_values);

        let sharpe = if !returns.is_empty() {
            calculate_sharpe_ratio(&returns)
        } else {
            0.0
        };

        let sortino = if !returns.is_empty() {
            calculate_sortino_ratio(&returns)
        } else {
            0.0
        };

        //trade statistics
        let trade_stats = calculate_trade_statistics(trades);

        //exposure calculation (simplified - percentage of time in market)
        let exposure = calculate_exposure(equity_curve, trades);

        SummaryMetrics {
            initial_balance,
            final_balance,
            total_return,
            total_return_pct,
            cagr,
            max_drawdown: max_dd,
            sharpe_ratio: sharpe,
            sortino_ratio: sortino,
            win_rate: trade_stats.win_rate,
            avg_win: trade_stats.avg_win,
            avg_loss: trade_stats.avg_loss,
            profit_factor: trade_stats.profit_factor,
            num_trades: trade_stats.num_trades,
            num_winning_trades: trade_stats.num_winning_trades,
            num_losing_trades: trade_stats.num_losing_trades,
            largest_win: trade_stats.largest_win,
            largest_loss: trade_stats.largest_loss,
            exposure,
        }
    }

    //prints metrics in a formatted table
    pub fn pretty_print_table(&self) {
        let mut table = Table::new();

        table.add_row(Row::new(vec![Cell::new("Metric"), Cell::new("Value")]));

        table.add_row(Row::new(vec![
            Cell::new("Initial Balance"),
            Cell::new(&format!("${:.2}", self.initial_balance)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Final Balance"),
            Cell::new(&format!("${:.2}", self.final_balance)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Total Return"),
            Cell::new(&format!(
                "${:.2} ({:.2}%)",
                self.total_return,
                self.total_return_pct * 100.0
            )),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("CAGR"),
            Cell::new(&format!("{:.2}%", self.cagr)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Max Drawdown"),
            Cell::new(&format!("{:.2}%", self.max_drawdown * 100.0)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Sharpe Ratio"),
            Cell::new(&format!("{:.3}", self.sharpe_ratio)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Sortino Ratio"),
            Cell::new(&format!("{:.3}", self.sortino_ratio)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Number of Trades"),
            Cell::new(&format!("{}", self.num_trades)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Win Rate"),
            Cell::new(&format!("{:.2}%", self.win_rate * 100.0)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Avg Win"),
            Cell::new(&format!("${:.2}", self.avg_win)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Avg Loss"),
            Cell::new(&format!("${:.2}", self.avg_loss)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Largest Win"),
            Cell::new(&format!("${:.2}", self.largest_win)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Largest Loss"),
            Cell::new(&format!("${:.2}", self.largest_loss)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Profit Factor"),
            Cell::new(&format!("{:.3}", self.profit_factor)),
        ]));

        table.add_row(Row::new(vec![
            Cell::new("Exposure"),
            Cell::new(&format!("{:.2}%", self.exposure * 100.0)),
        ]));

        table.printstd();
    }
}

struct TradeStats {
    num_trades: usize,
    num_winning_trades: usize,
    num_losing_trades: usize,
    win_rate: f64,
    avg_win: f64,
    avg_loss: f64,
    profit_factor: f64,
    largest_win: f64,
    largest_loss: f64,
}

fn calculate_trade_statistics(trades: &[Fill]) -> TradeStats {
    if trades.is_empty() {
        return TradeStats {
            num_trades: 0,
            num_winning_trades: 0,
            num_losing_trades: 0,
            win_rate: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            profit_factor: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
        };
    }

    //group trades into round trips (open + close)
    let mut round_trips = Vec::new();
    let mut open_trades: Vec<&Fill> = Vec::new();

    for trade in trades {
        if open_trades.is_empty() {
            open_trades.push(trade);
        } else {
            let last = open_trades.last().unwrap();
            let same_direction = (last.qty > 0 && trade.qty > 0) || (last.qty < 0 && trade.qty < 0);

            if same_direction {
                open_trades.push(trade);
            } else {
                //closing trade - calculate pnl
                let total_qty = open_trades.iter().map(|t| t.qty.abs()).sum::<i32>();
                let avg_entry = open_trades
                    .iter()
                    .map(|t| t.fill_price * t.qty.abs() as f64)
                    .sum::<f64>()
                    / total_qty as f64;

                let profit_loss = if open_trades[0].qty > 0 {
                    //long trade
                    (trade.fill_price - avg_entry) * total_qty.min(trade.qty.abs()) as f64
                } else {
                    //short trade
                    (avg_entry - trade.fill_price) * total_qty.min(trade.qty.abs()) as f64
                };

                round_trips.push(profit_loss);

                //if trade closes more than open position, it opens a new one
                if trade.qty.abs() > total_qty {
                    open_trades.clear();
                    open_trades.push(trade);
                } else {
                    open_trades.clear();
                }
            }
        }
    }

    if round_trips.is_empty() {
        return TradeStats {
            num_trades: 0,
            num_winning_trades: 0,
            num_losing_trades: 0,
            win_rate: 0.0,
            avg_win: 0.0,
            avg_loss: 0.0,
            profit_factor: 0.0,
            largest_win: 0.0,
            largest_loss: 0.0,
        };
    }

    let winning_trades: Vec<f64> = round_trips
        .iter()
        .filter(|&&profit_loss| profit_loss > 0.0)
        .copied()
        .collect();
    let losing_trades: Vec<f64> = round_trips
        .iter()
        .filter(|&&profit_loss| profit_loss < 0.0)
        .copied()
        .collect();

    let num_winning = winning_trades.len();
    let num_losing = losing_trades.len();
    let total = round_trips.len();

    let win_rate = num_winning as f64 / total as f64;

    let avg_win = if num_winning > 0 {
        winning_trades.iter().sum::<f64>() / num_winning as f64
    } else {
        0.0
    };

    let avg_loss = if num_losing > 0 {
        losing_trades.iter().sum::<f64>() / num_losing as f64
    } else {
        0.0
    };

    let total_wins: f64 = winning_trades.iter().sum();
    let total_losses: f64 = losing_trades.iter().sum::<f64>().abs();

    let profit_factor = if total_losses > 0.0 {
        total_wins / total_losses
    } else if total_wins > 0.0 {
        f64::INFINITY
    } else {
        0.0
    };

    let largest_win = winning_trades.iter().fold(0.0f64, |a, &b| a.max(b));
    let largest_loss = losing_trades.iter().fold(0.0f64, |a, &b| a.min(b));

    TradeStats {
        num_trades: total,
        num_winning_trades: num_winning,
        num_losing_trades: num_losing,
        win_rate,
        avg_win,
        avg_loss,
        profit_factor,
        largest_win,
        largest_loss,
    }
}

fn calculate_sharpe_ratio(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean = returns.mean();
    let std_dev = returns.std_dev();

    if std_dev == 0.0 {
        return 0.0;
    }

    //annualize assuming daily returns
    //sharpe = (mean_return * 252) / (std_dev * sqrt(252))
    //simplified sharpe = mean / std_dev * sqrt(252)
    (mean / std_dev) * (252.0_f64).sqrt()
}

fn calculate_sortino_ratio(returns: &[f64]) -> f64 {
    if returns.is_empty() {
        return 0.0;
    }

    let mean = returns.mean();

    //calculate downside deviation (only negative returns)
    let negative_returns: Vec<f64> = returns.iter().filter(|&&r| r < 0.0).copied().collect();

    if negative_returns.is_empty() {
        return if mean > 0.0 { f64::INFINITY } else { 0.0 };
    }

    let downside_dev = negative_returns.std_dev();

    if downside_dev == 0.0 {
        return 0.0;
    }

    //annualize
    (mean / downside_dev) * (252.0_f64).sqrt()
}

fn calculate_exposure(equity_curve: &[EquityPoint], trades: &[Fill]) -> f64 {
    if equity_curve.len() < 2 {
        return 0.0;
    }

    //track position over time
    let mut in_market_count = 0;
    let mut current_position = 0i32;

    let mut trade_idx = 0;

    for point in equity_curve {
        //update position based on trades up to this timestamp
        while trade_idx < trades.len() && trades[trade_idx].timestamp <= point.timestamp {
            current_position += trades[trade_idx].qty;
            trade_idx += 1;
        }

        if current_position != 0 {
            in_market_count += 1;
        }
    }

    in_market_count as f64 / equity_curve.len() as f64
}
