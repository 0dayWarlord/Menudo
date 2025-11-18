use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use menudo::prelude::*;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "menudo")]
#[command(about = "A Rust-based strategy backtesting engine for futures", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    //run a backtest
    Run {
        //path to csv data file
        #[arg(long)]
        data: PathBuf,

        //symbol to trade (eg es, nq)
        #[arg(long)]
        symbol: String,

        //strategy type (sma, rsi)
        #[arg(long)]
        strategy: String,

        //contract month (eg 2025-03)
        #[arg(long, default_value = "2025-03")]
        contract_month: String,

        //tick size
        #[arg(long)]
        tick_size: f64,

        //tick value (dollar value of one tick)
        #[arg(long)]
        tick_value: f64,

        //point value (optional, defaults to tick_value/tick_size)
        #[arg(long)]
        point_value: Option<f64>,

        //initial margin per contract (optional)
        #[arg(long)]
        initial_margin: Option<f64>,

        //maintenance margin per contract (optional)
        #[arg(long)]
        maintenance_margin: Option<f64>,

        //initial account balance
        #[arg(long, default_value = "100000")]
        initial_balance: f64,

        //commission per contract per side
        #[arg(long, default_value = "2.5")]
        commission: f64,

        //slippage per contract per side
        #[arg(long, default_value = "1.0")]
        slippage: f64,

        //sma strategy parameters
        //fast sma window (for sma strategy)
        #[arg(long)]
        fast: Option<usize>,

        //slow sma window (for sma strategy)
        #[arg(long)]
        slow: Option<usize>,

        //rsi strategy parameters
        //rsi lookback period (for rsi strategy)
        #[arg(long)]
        rsi_lookback: Option<usize>,

        //rsi lower threshold (for rsi strategy)
        #[arg(long)]
        rsi_lower: Option<f64>,

        //rsi upper threshold (for rsi strategy)
        #[arg(long)]
        rsi_upper: Option<f64>,

        //common strategy parameter
        //number of contracts to trade
        #[arg(long, default_value = "1")]
        qty: u32,

        //output options
        //output path for equity curve csv
        #[arg(long)]
        output_equity_csv: Option<PathBuf>,

        //output path for trades csv
        #[arg(long)]
        output_trades_csv: Option<PathBuf>,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Run {
            data,
            symbol,
            strategy,
            contract_month,
            tick_size,
            tick_value,
            point_value,
            initial_margin,
            maintenance_margin,
            initial_balance,
            commission,
            slippage,
            fast,
            slow,
            rsi_lookback,
            rsi_lower,
            rsi_upper,
            qty,
            output_equity_csv,
            output_trades_csv,
        } => {
            run_backtest(
                data,
                symbol,
                strategy,
                contract_month,
                tick_size,
                tick_value,
                point_value,
                initial_margin,
                maintenance_margin,
                initial_balance,
                commission,
                slippage,
                fast,
                slow,
                rsi_lookback,
                rsi_lower,
                rsi_upper,
                qty,
                output_equity_csv,
                output_trades_csv,
            )?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_backtest(
    data_path: PathBuf,
    symbol: String,
    strategy_name: String,
    contract_month: String,
    tick_size: f64,
    tick_value: f64,
    point_value: Option<f64>,
    initial_margin: Option<f64>,
    maintenance_margin: Option<f64>,
    initial_balance: f64,
    commission: f64,
    slippage: f64,
    fast: Option<usize>,
    slow: Option<usize>,
    rsi_lookback: Option<usize>,
    rsi_lower: Option<f64>,
    rsi_upper: Option<f64>,
    qty: u32,
    output_equity_csv: Option<PathBuf>,
    output_trades_csv: Option<PathBuf>,
) -> Result<()> {
    println!("Menudo Futures Backtesting Engine");
    println!("==================================\n");

    //load data
    println!("Loading data from {:?}...", data_path);
    let all_bars =
        load_csv(&data_path).context(format!("Failed to load data from {:?}", data_path))?;

    //filter by symbol
    let bars = filter_by_symbol(&all_bars, &symbol);

    if bars.is_empty() {
        anyhow::bail!("No data found for symbol {}", symbol);
    }

    println!("Loaded {} bars for {}", bars.len(), symbol);
    println!(
        "Date range: {} to {}\n",
        bars.first().unwrap().timestamp,
        bars.last().unwrap().timestamp
    );

    //create contract
    let contract = FuturesContract::from_params(
        symbol.clone(),
        contract_month,
        tick_size,
        tick_value,
        point_value,
        initial_margin,
        maintenance_margin,
    );

    println!(
        "Contract: {} (tick: ${}, value: ${})",
        contract.symbol, contract.tick_size, contract.tick_value
    );
    println!("Initial margin: ${:.2}\n", contract.initial_margin);

    //create strategy
    let strategy_type = StrategyType::parse(&strategy_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown strategy: {}", strategy_name))?;

    let mut strategy: Box<dyn Strategy> = match strategy_type {
        StrategyType::SmaCrossover => {
            let fast = fast.ok_or_else(|| anyhow::anyhow!("--fast required for SMA strategy"))?;
            let slow = slow.ok_or_else(|| anyhow::anyhow!("--slow required for SMA strategy"))?;

            println!("Strategy: SMA Crossover (fast={}, slow={})", fast, slow);
            Box::new(SmaCrossoverStrategy::new(symbol.clone(), fast, slow, qty))
        }
        StrategyType::RsiReversion => {
            let lookback = rsi_lookback.unwrap_or(14);
            let lower = rsi_lower.unwrap_or(30.0);
            let upper = rsi_upper.unwrap_or(70.0);

            println!(
                "Strategy: RSI Reversion (lookback={}, lower={}, upper={})",
                lookback, lower, upper
            );
            Box::new(RsiReversionStrategy::new(
                symbol.clone(),
                lookback,
                lower,
                upper,
                qty,
            ))
        }
    };

    println!("Quantity: {} contract(s)", qty);
    println!("Initial balance: ${:.2}", initial_balance);
    println!("Commission: ${:.2} per contract", commission);
    println!("Slippage: ${:.2} per contract\n", slippage);

    //create backtest config
    let config = BacktestConfig {
        initial_balance,
        commission_per_contract: commission,
        slippage_per_contract: slippage,
        max_lookback: 500,
    };

    //run backtest
    println!("Running backtest...\n");
    let mut engine = BacktestEngine::new(config, bars, contract);
    let result = engine.run(&mut strategy);

    //display results
    println!("Backtest Results");
    println!("================\n");
    result.summary.pretty_print_table();

    //save outputs if requested
    if let Some(equity_path) = output_equity_csv {
        save_equity_csv(&result.equity_curve, &equity_path)?;
        println!("\nEquity curve saved to {:?}", equity_path);
    }

    if let Some(trades_path) = output_trades_csv {
        save_trades_csv(&result.trades, &trades_path)?;
        println!("Trades saved to {:?}", trades_path);
    }

    Ok(())
}

fn save_equity_csv(equity_curve: &[EquityPoint], path: &PathBuf) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;
    writeln!(file, "timestamp,equity,drawdown,returns")?;

    for point in equity_curve {
        writeln!(
            file,
            "{},{},{},{}",
            point.timestamp.to_rfc3339(),
            point.equity,
            point.drawdown,
            point.returns
        )?;
    }

    Ok(())
}

fn save_trades_csv(trades: &[Fill], path: &PathBuf) -> Result<()> {
    use std::io::Write;

    let mut file = std::fs::File::create(path)?;
    writeln!(
        file,
        "id,order_id,timestamp,symbol,qty,side,fill_price,fees"
    )?;

    for trade in trades {
        writeln!(
            file,
            "{},{},{},{},{},{:?},{},{}",
            trade.id,
            trade.order_id,
            trade.timestamp.to_rfc3339(),
            trade.symbol,
            trade.qty,
            trade.side,
            trade.fill_price,
            trade.fees
        )?;
    }

    Ok(())
}
