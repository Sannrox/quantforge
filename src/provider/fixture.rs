use std::path::Path;

use serde::Deserialize;

use crate::domain::{Financials, Ohlcv, Quote};
use crate::error::AppError;

#[derive(Debug, Deserialize)]
struct FixtureFile {
    quote: Quote,
    annual: Vec<Financials>,
    quarterly: Vec<Financials>,
    #[serde(default)]
    prices: Vec<Ohlcv>,
}

fn load(dir: &Path, ticker: &str) -> Result<FixtureFile, AppError> {
    let path = dir.join(format!("{}.json", ticker.to_ascii_lowercase()));
    let bytes = std::fs::read(&path).map_err(|_| {
        AppError::NotFound(format!(
            "fixture has no data for {ticker}. Add ACME for the offline demo, or switch the provider to yahoo in Settings"
        ))
    })?;
    serde_json::from_slice(&bytes).map_err(AppError::from)
}

pub fn quote(dir: &Path, ticker: &str) -> Result<Quote, AppError> {
    let mut quote = load(dir, ticker)?.quote;
    quote.ticker = ticker.to_string();
    Ok(quote)
}

pub fn financials(dir: &Path, ticker: &str, annual: bool) -> Result<Vec<Financials>, AppError> {
    let file = load(dir, ticker)?;
    Ok(if annual { file.annual } else { file.quarterly })
}

pub fn prices(dir: &Path, ticker: &str) -> Result<Vec<Ohlcv>, AppError> {
    let file = load(dir, ticker)?;
    if !file.prices.is_empty() {
        return Ok(file.prices);
    }
    Ok(synthesize_prices(&file.annual))
}

fn synthesize_prices(annual: &[Financials]) -> Vec<Ohlcv> {
    let mut bars = Vec::new();
    let chronological: Vec<&Financials> = annual.iter().rev().collect();
    for window in chronological.windows(2) {
        let start = window[0];
        let end = window[1];
        let start_price = start.year_end_price.unwrap_or(0.0);
        let end_price = end.year_end_price.unwrap_or(start_price);
        if start_price <= 0.0 && end_price <= 0.0 {
            continue;
        }
        let start_year: i32 = start
            .period_end
            .get(0..4)
            .and_then(|y| y.parse().ok())
            .unwrap_or(2000);
        for month in 1..=12 {
            let t = f64::from(month) / 12.0;
            let price = start_price + (end_price - start_price) * t;
            bars.push(Ohlcv {
                date: format!("{start_year}-{month:02}-28"),
                close: price,
                adj_close: Some(price),
            });
        }
    }
    if let Some(last) = chronological.last() {
        if let Some(price) = last.year_end_price {
            bars.push(Ohlcv {
                date: last.period_end.clone(),
                close: price,
                adj_close: Some(price),
            });
        }
    }
    bars
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn testdata() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")
    }

    #[test]
    fn loads_acme() {
        let quote = quote(&testdata(), "ACME").expect("acme");
        assert_eq!(quote.ticker, "ACME");
        assert!(quote.price > 0.0);
        let annual = financials(&testdata(), "ACME", true).expect("annual");
        assert!(annual.len() >= 10);
        let prices = prices(&testdata(), "ACME").expect("prices");
        assert!(!prices.is_empty());
    }
}
