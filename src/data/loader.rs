use crate::data::bar::Bar;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use csv::ReaderBuilder;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct CsvRecord {
    timestamp: String,
    open: f64,
    high: f64,
    low: f64,
    close: f64,
    volume: f64,
    #[serde(default)]
    open_interest: Option<f64>,
    symbol: String,
}

//loads bars from a csv file
pub fn load_csv<P: AsRef<Path>>(path: P) -> Result<Vec<Bar>> {
    let path = path.as_ref();
    let mut reader = ReaderBuilder::new()
        .has_headers(true)
        .from_path(path)
        .context(format!("Failed to open CSV file: {:?}", path))?;

    let mut bars = Vec::new();

    for (index, result) in reader.deserialize().enumerate() {
        let record: CsvRecord =
            result.context(format!("Failed to parse CSV record at line {}", index + 2))?;

        //parse timestamp
        let timestamp = DateTime::parse_from_rfc3339(&record.timestamp)
            .context(format!(
                "Failed to parse timestamp '{}' at line {}",
                record.timestamp,
                index + 2
            ))?
            .with_timezone(&Utc);

        //create bar
        let bar = Bar::new_unchecked(
            timestamp,
            record.open,
            record.high,
            record.low,
            record.close,
            record.volume,
            record.open_interest,
            record.symbol,
        );

        bars.push(bar);
    }

    //sort by timestamp to ensure chronological order
    bars.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

    Ok(bars)
}

//filters bars by symbol
pub fn filter_by_symbol(bars: &[Bar], symbol: &str) -> Vec<Bar> {
    bars.iter()
        .filter(|bar| bar.symbol == symbol)
        .cloned()
        .collect()
}
