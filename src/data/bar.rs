use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BarError {
    #[error("Invalid OHLC values: high ({high}) < low ({low})")]
    InvalidHighLow { high: f64, low: f64 },
    #[error("Invalid OHLC values: close ({close}) outside high-low range [{low}, {high}]")]
    InvalidClose { close: f64, high: f64, low: f64 },
    #[error("Invalid OHLC values: open ({open}) outside high-low range [{low}, {high}]")]
    InvalidOpen { open: f64, high: f64, low: f64 },
    #[error("Negative volume: {0}")]
    NegativeVolume(f64),
}

//represents a single ohlcv bar (candlestick) of market data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Bar {
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub open_interest: Option<f64>,
    pub symbol: String,
}

impl Bar {
    //creates a new Bar with validation
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        open_interest: Option<f64>,
        symbol: String,
    ) -> Result<Self, BarError> {
        //validate high >= low
        if high < low {
            return Err(BarError::InvalidHighLow { high, low });
        }

        //validate close within [low, high]
        if close < low || close > high {
            return Err(BarError::InvalidClose { close, high, low });
        }

        //validate open within [low, high]
        if open < low || open > high {
            return Err(BarError::InvalidOpen { open, high, low });
        }

        //validate non-negative volume
        if volume < 0.0 {
            return Err(BarError::NegativeVolume(volume));
        }

        Ok(Bar {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            open_interest,
            symbol,
        })
    }

    //creates a Bar without validation
    #[allow(clippy::too_many_arguments)]
    pub fn new_unchecked(
        timestamp: DateTime<Utc>,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        volume: f64,
        open_interest: Option<f64>,
        symbol: String,
    ) -> Self {
        Bar {
            timestamp,
            open,
            high,
            low,
            close,
            volume,
            open_interest,
            symbol,
        }
    }

    //returns the typical price (HLC/3)
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    //returns the mid price ((high + low) / 2)
    pub fn mid_price(&self) -> f64 {
        (self.high + self.low) / 2.0
    }

    //returns the range (high - low)
    pub fn range(&self) -> f64 {
        self.high - self.low
    }
}
