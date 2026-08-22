use reqwest::Client;
use serde::Deserialize;

use crate::domain::{Financials, Ohlcv, Quote};
use crate::error::AppError;

#[derive(Debug, Deserialize)]
struct FmpQuote {
    symbol: Option<String>,
    name: Option<String>,
    price: Option<f64>,
    #[serde(default, rename = "marketCap")]
    market_cap: Option<f64>,
    #[serde(default)]
    currency: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FmpProfile {
    sector: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FmpIncome {
    date: Option<String>,
    period: Option<String>,
    reported_currency: Option<String>,
    revenue: Option<f64>,
    ebitda: Option<f64>,
    gross_profit: Option<f64>,
    operating_income: Option<f64>,
    net_income: Option<f64>,
    epsdiluted: Option<f64>,
    weighted_average_shs_out_dil: Option<f64>,
    income_before_tax: Option<f64>,
    income_tax_expense: Option<f64>,
    interest_expense: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FmpCash {
    date: Option<String>,
    operating_cash_flow: Option<f64>,
    free_cash_flow: Option<f64>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FmpBalance {
    date: Option<String>,
    cash_and_short_term_investments: Option<f64>,
    cash_and_cash_equivalents: Option<f64>,
    short_term_investments: Option<f64>,
    total_debt: Option<f64>,
    short_term_debt: Option<f64>,
    long_term_debt: Option<f64>,
    total_stockholders_equity: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct FmpHistory {
    historical: Option<Vec<FmpBar>>,
}

#[derive(Debug, Deserialize)]
struct FmpBar {
    date: Option<String>,
    close: Option<f64>,
    #[serde(default, rename = "adjClose")]
    adj_close: Option<f64>,
}

fn require_key(key: Option<&str>) -> Result<&str, AppError> {
    key.filter(|value| !value.is_empty())
        .ok_or_else(|| AppError::BadRequest("FMP provider needs an API key in Settings".into()))
}

pub async fn quote(http: &Client, key: Option<&str>, ticker: &str) -> Result<Quote, AppError> {
    let key = require_key(key)?;
    let quotes: Vec<FmpQuote> = get(
        http,
        &format!("https://financialmodelingprep.com/api/v3/quote/{ticker}?apikey={key}"),
    )
    .await?;
    let quote = quotes
        .into_iter()
        .next()
        .ok_or_else(|| AppError::Provider(format!("fmp: no quote for {ticker}")))?;
    let profiles: Vec<FmpProfile> = get(
        http,
        &format!("https://financialmodelingprep.com/api/v3/profile/{ticker}?apikey={key}"),
    )
    .await
    .unwrap_or_default();
    Ok(Quote {
        ticker: quote.symbol.unwrap_or_else(|| ticker.to_string()),
        name: quote.name.unwrap_or_else(|| ticker.to_string()),
        sector: profiles
            .first()
            .and_then(|row| row.sector.clone())
            .unwrap_or_default(),
        price: quote
            .price
            .ok_or_else(|| AppError::Provider(format!("fmp: no price for {ticker}")))?,
        currency: quote.currency.unwrap_or_else(|| "USD".into()),
        market_cap: quote.market_cap,
        shares_outstanding: None,
    })
}

pub async fn financials(
    http: &Client,
    key: Option<&str>,
    ticker: &str,
    annual: bool,
) -> Result<Vec<Financials>, AppError> {
    let key = require_key(key)?;
    let period = if annual { "annual" } else { "quarter" };
    let income: Vec<FmpIncome> = get(
        http,
        &format!(
            "https://financialmodelingprep.com/api/v3/income-statement/{ticker}?period={period}&limit=40&apikey={key}"
        ),
    )
    .await?;
    let cash: Vec<FmpCash> = get(
        http,
        &format!(
            "https://financialmodelingprep.com/api/v3/cash-flow-statement/{ticker}?period={period}&limit=40&apikey={key}"
        ),
    )
    .await?;
    let balance: Vec<FmpBalance> = get(
        http,
        &format!(
            "https://financialmodelingprep.com/api/v3/balance-sheet-statement/{ticker}?period={period}&limit=40&apikey={key}"
        ),
    )
    .await
    .unwrap_or_default();
    if income.is_empty() {
        return Err(AppError::Provider(format!(
            "fmp: no statements for {ticker}"
        )));
    }
    Ok(income
        .into_iter()
        .map(|row| {
            let date = row.date.clone().unwrap_or_default();
            let cash_row = cash
                .iter()
                .find(|item| item.date.as_deref() == Some(date.as_str()));
            let balance_row = balance
                .iter()
                .find(|item| item.date.as_deref() == Some(date.as_str()));
            Financials {
                period_end: date,
                fiscal_period: row
                    .period
                    .unwrap_or_else(|| if annual { "FY".into() } else { "Q".into() }),
                currency: row.reported_currency.unwrap_or_else(|| "USD".into()),
                revenue: row.revenue,
                ebitda: row.ebitda,
                gross_profit: row.gross_profit,
                operating_income: row.operating_income,
                net_income: row.net_income,
                operating_cash_flow: cash_row.and_then(|item| item.operating_cash_flow),
                free_cash_flow: cash_row.and_then(|item| item.free_cash_flow),
                eps: row.epsdiluted,
                shares_outstanding: row.weighted_average_shs_out_dil,
                year_end_price: None,
                cash: balance_row.and_then(cash_like),
                debt: balance_row.and_then(interest_bearing_debt),
                equity: balance_row.and_then(|item| item.total_stockholders_equity),
                pretax_income: row.income_before_tax,
                tax_expense: row.income_tax_expense,
                interest_expense: row
                    .interest_expense
                    .map(f64::abs)
                    .filter(|value| *value > 0.0),
            }
        })
        .collect())
}

fn cash_like(row: &FmpBalance) -> Option<f64> {
    row.cash_and_short_term_investments
        .or_else(|| sum_opt(row.cash_and_cash_equivalents, row.short_term_investments))
        .or(row.cash_and_cash_equivalents)
}

fn interest_bearing_debt(row: &FmpBalance) -> Option<f64> {
    row.total_debt
        .or_else(|| sum_opt(row.short_term_debt, row.long_term_debt))
}

fn sum_opt(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

pub async fn prices(
    http: &Client,
    key: Option<&str>,
    ticker: &str,
) -> Result<Vec<Ohlcv>, AppError> {
    let key = require_key(key)?;
    let history: FmpHistory = get(
        http,
        &format!(
            "https://financialmodelingprep.com/api/v3/historical-price-full/{ticker}?apikey={key}"
        ),
    )
    .await?;
    let mut bars: Vec<Ohlcv> = history
        .historical
        .unwrap_or_default()
        .into_iter()
        .filter_map(|bar| {
            Some(Ohlcv {
                date: bar.date?,
                close: bar.close?,
                adj_close: bar.adj_close,
            })
        })
        .collect();
    bars.reverse();
    if bars.is_empty() {
        return Err(AppError::Provider(format!("fmp: no prices for {ticker}")));
    }
    Ok(bars)
}

async fn get<T: serde::de::DeserializeOwned>(http: &Client, url: &str) -> Result<T, AppError> {
    let response = http.get(url).send().await?;
    if !response.status().is_success() {
        return Err(AppError::Provider(format!(
            "fmp {}: {}",
            request_target(url),
            response.status()
        )));
    }
    Ok(response.json().await?)
}

fn request_target(url: &str) -> &str {
    url.split_once('?').map(|(path, _)| path).unwrap_or(url)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_errors_omit_api_key() {
        let url = "https://financialmodelingprep.com/api/v3/quote/ACME?apikey=super-secret";
        let target = request_target(url);
        assert!(!target.contains("super-secret"));
        assert!(!target.contains("apikey"));
        assert_eq!(
            target,
            "https://financialmodelingprep.com/api/v3/quote/ACME"
        );
    }
}
