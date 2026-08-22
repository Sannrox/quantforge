use std::collections::{BTreeMap, HashMap};

use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::domain::{Financials, Ohlcv, Quote};
use crate::error::AppError;

const TIMESERIES_START: i64 = 1_483_142_400; // 2016-12-31 UTC; Yahoo still caps depth.

const ANNUAL_TYPES: &[&str] = &[
    "annualTotalRevenue",
    "annualNetIncome",
    "annualOperatingIncome",
    "annualGrossProfit",
    "annualEBITDA",
    "annualNormalizedEBITDA",
    "annualOperatingCashFlow",
    "annualCapitalExpenditure",
    "annualFreeCashFlow",
    "annualDilutedAverageShares",
    "annualOrdinarySharesNumber",
    "annualBasicAverageShares",
    "annualDilutedEPS",
    "annualCashCashEquivalentsAndShortTermInvestments",
    "annualCashAndCashEquivalents",
    "annualTotalDebt",
    "annualStockholdersEquity",
    "annualPretaxIncome",
    "annualTaxProvision",
    "annualInterestExpense",
    "annualInterestExpenseNonOperating",
];

const QUARTERLY_TYPES: &[&str] = &[
    "quarterlyTotalRevenue",
    "quarterlyNetIncome",
    "quarterlyOperatingIncome",
    "quarterlyGrossProfit",
    "quarterlyEBITDA",
    "quarterlyNormalizedEBITDA",
    "quarterlyOperatingCashFlow",
    "quarterlyCapitalExpenditure",
    "quarterlyFreeCashFlow",
    "quarterlyDilutedAverageShares",
    "quarterlyOrdinarySharesNumber",
    "quarterlyBasicAverageShares",
    "quarterlyDilutedEPS",
    "quarterlyCashCashEquivalentsAndShortTermInvestments",
    "quarterlyCashAndCashEquivalents",
    "quarterlyTotalDebt",
    "quarterlyStockholdersEquity",
    "quarterlyPretaxIncome",
    "quarterlyTaxProvision",
    "quarterlyInterestExpense",
    "quarterlyInterestExpenseNonOperating",
];

#[derive(Debug, Deserialize)]
struct ChartResponse {
    chart: ChartBody,
}

#[derive(Debug, Deserialize)]
struct ChartBody {
    result: Option<Vec<ChartResult>>,
    error: Option<Value>,
}

#[derive(Debug, Deserialize)]
struct ChartResult {
    meta: ChartMeta,
    timestamp: Option<Vec<i64>>,
    indicators: ChartIndicators,
}

#[derive(Debug, Deserialize)]
struct ChartMeta {
    #[serde(default, rename = "shortName")]
    short_name: Option<String>,
    #[serde(default, rename = "longName")]
    long_name: Option<String>,
    #[serde(default, rename = "regularMarketPrice")]
    regular_market_price: Option<f64>,
    #[serde(default, rename = "chartPreviousClose")]
    chart_previous_close: Option<f64>,
    #[serde(default)]
    currency: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ChartIndicators {
    quote: Option<Vec<ChartQuote>>,
    #[serde(default, rename = "adjclose")]
    adj_close: Option<Vec<ChartAdj>>,
}

#[derive(Debug, Deserialize)]
struct ChartQuote {
    close: Option<Vec<Option<f64>>>,
}

#[derive(Debug, Deserialize)]
struct ChartAdj {
    adjclose: Option<Vec<Option<f64>>>,
}

#[derive(Debug, Deserialize)]
struct SearchResponse {
    quotes: Option<Vec<SearchQuote>>,
}

#[derive(Debug, Deserialize)]
struct SearchQuote {
    symbol: Option<String>,
    #[serde(default, rename = "shortname")]
    short_name: Option<String>,
    #[serde(default, rename = "longname")]
    long_name: Option<String>,
    #[serde(default)]
    sector: Option<String>,
}

#[derive(Debug, Default)]
struct PeriodFacts {
    currency: String,
    fields: HashMap<String, f64>,
}

pub async fn quote(http: &Client, ticker: &str) -> Result<Quote, AppError> {
    let yahoo = yahoo_symbol(ticker);
    let chart = chart(http, &yahoo, "1y").await?;
    let result = chart_result(&chart)?;
    let price = result
        .meta
        .regular_market_price
        .or(result.meta.chart_previous_close)
        .ok_or_else(|| AppError::Provider(format!("yahoo: no price for {ticker}")))?;
    let hit = search(http, ticker).await.ok().and_then(|rows| {
        rows.into_iter().find(|row| {
            row.symbol.as_deref().is_some_and(|symbol| {
                symbol.eq_ignore_ascii_case(ticker) || symbol.eq_ignore_ascii_case(&yahoo)
            })
        })
    });
    let name = result
        .meta
        .long_name
        .clone()
        .or_else(|| result.meta.short_name.clone())
        .or_else(|| hit.as_ref().and_then(|row| row.long_name.clone()))
        .or_else(|| hit.as_ref().and_then(|row| row.short_name.clone()))
        .unwrap_or_else(|| ticker.to_string());
    let sector = hit
        .as_ref()
        .and_then(|row| row.sector.clone())
        .unwrap_or_default();
    Ok(Quote {
        ticker: ticker.to_string(),
        name,
        sector,
        price,
        currency: result.meta.currency.clone().unwrap_or_else(|| "USD".into()),
        market_cap: None,
        shares_outstanding: None,
    })
}

pub async fn financials(
    http: &Client,
    ticker: &str,
    annual: bool,
) -> Result<Vec<Financials>, AppError> {
    let symbol = yahoo_symbol(ticker);
    let types = if annual {
        ANNUAL_TYPES
    } else {
        QUARTERLY_TYPES
    };
    let payload = timeseries(http, &symbol, types).await?;
    let rows = rows_from_timeseries(&payload, annual);
    if rows.is_empty() {
        return Err(AppError::Provider(format!(
            "yahoo: no {} statements for {ticker}",
            if annual { "annual" } else { "quarterly" }
        )));
    }
    Ok(rows)
}

pub async fn prices(http: &Client, ticker: &str) -> Result<Vec<Ohlcv>, AppError> {
    let symbol = yahoo_symbol(ticker);
    let chart = chart(http, &symbol, "max").await?;
    let result = chart_result(&chart)?;
    let timestamps = result.timestamp.clone().unwrap_or_default();
    let closes = result
        .indicators
        .quote
        .as_ref()
        .and_then(|quotes| quotes.first())
        .and_then(|quote| quote.close.clone())
        .unwrap_or_default();
    let adj = result
        .indicators
        .adj_close
        .as_ref()
        .and_then(|rows| rows.first())
        .and_then(|row| row.adjclose.clone())
        .unwrap_or_default();
    let mut bars = Vec::new();
    for (index, ts) in timestamps.iter().enumerate() {
        let close = closes.get(index).and_then(|value| *value);
        let Some(close) = close else {
            continue;
        };
        let adj_close = adj.get(index).and_then(|value| *value);
        bars.push(Ohlcv {
            date: unix_date(*ts),
            close,
            adj_close,
        });
    }
    if bars.is_empty() {
        return Err(AppError::Provider(format!("yahoo: no prices for {ticker}")));
    }
    Ok(bars)
}

async fn chart(http: &Client, ticker: &str, range: &str) -> Result<ChartResponse, AppError> {
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{ticker}?range={range}&interval=1d&events=div"
    );
    let response = yahoo_get(http, &url).await?;
    if !response.status().is_success() {
        return Err(AppError::Provider(format!(
            "yahoo chart {}: {}",
            ticker,
            response.status()
        )));
    }
    Ok(response.json().await?)
}

async fn search(http: &Client, ticker: &str) -> Result<Vec<SearchQuote>, AppError> {
    let url = format!("https://query1.finance.yahoo.com/v1/finance/search?q={ticker}");
    let response = yahoo_get(http, &url).await?;
    if !response.status().is_success() {
        return Err(AppError::Provider(format!(
            "yahoo search {}: {}",
            ticker,
            response.status()
        )));
    }
    let body: SearchResponse = response.json().await?;
    Ok(body.quotes.unwrap_or_default())
}

async fn timeseries(http: &Client, ticker: &str, types: &[&str]) -> Result<Value, AppError> {
    let period2 = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|elapsed| elapsed.as_secs() as i64)
        .unwrap_or(TIMESERIES_START + 1);
    let url = format!(
        "https://query1.finance.yahoo.com/ws/fundamentals-timeseries/v1/finance/timeseries/{ticker}?symbol={ticker}&type={}&period1={TIMESERIES_START}&period2={period2}",
        types.join(",")
    );
    let response = yahoo_get(http, &url).await?;
    if !response.status().is_success() {
        return Err(AppError::Provider(format!(
            "yahoo timeseries {}: {}",
            ticker,
            response.status()
        )));
    }
    Ok(response.json().await?)
}

async fn yahoo_get(http: &Client, url: &str) -> Result<reqwest::Response, AppError> {
    Ok(http.get(url).send().await?)
}

fn chart_result(chart: &ChartResponse) -> Result<&ChartResult, AppError> {
    if let Some(error) = &chart.chart.error {
        return Err(AppError::Provider(format!("yahoo: {error}")));
    }
    chart
        .chart
        .result
        .as_ref()
        .and_then(|rows| rows.first())
        .ok_or_else(|| AppError::Provider("yahoo: empty chart".into()))
}

fn rows_from_timeseries(payload: &Value, annual: bool) -> Vec<Financials> {
    let mut by_date: BTreeMap<String, PeriodFacts> = BTreeMap::new();
    let Some(results) = payload
        .pointer("/timeseries/result")
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    for series in results {
        let Some(full_name) = series.pointer("/meta/type/0").and_then(Value::as_str) else {
            continue;
        };
        let name = unprefixed(full_name);
        let Some(points) = series.get(full_name).and_then(Value::as_array) else {
            continue;
        };
        for point in points {
            let Some(end) = point.get("asOfDate").and_then(Value::as_str) else {
                continue;
            };
            let Some(value) = point.pointer("/reportedValue/raw").and_then(json_f64) else {
                continue;
            };
            let facts = by_date.entry(end.to_string()).or_default();
            if facts.currency.is_empty() {
                facts.currency = point
                    .get("currencyCode")
                    .and_then(Value::as_str)
                    .unwrap_or("USD")
                    .to_string();
            }
            facts.fields.insert(name.to_string(), value);
        }
    }
    by_date
        .into_iter()
        .rev()
        .map(|(period_end, facts)| financials_from_facts(period_end, facts, annual))
        .collect()
}

fn financials_from_facts(period_end: String, facts: PeriodFacts, annual: bool) -> Financials {
    let operating_cf = first(&facts, &["OperatingCashFlow"]);
    let capex = first(&facts, &["CapitalExpenditure"]);
    let fcf = first(&facts, &["FreeCashFlow"]).or_else(|| match (operating_cf, capex) {
        (Some(ocf), Some(capex)) => Some(ocf + capex),
        (Some(ocf), None) => Some(ocf),
        _ => None,
    });
    let currency = if facts.currency.is_empty() {
        "USD".into()
    } else {
        facts.currency.clone()
    };
    Financials {
        fiscal_period: period_label(annual, &period_end),
        currency,
        revenue: first(&facts, &["TotalRevenue"]),
        ebitda: first(&facts, &["EBITDA", "NormalizedEBITDA"]),
        gross_profit: first(&facts, &["GrossProfit"]),
        operating_income: first(&facts, &["OperatingIncome"]),
        net_income: first(&facts, &["NetIncome"]),
        operating_cash_flow: operating_cf,
        free_cash_flow: fcf,
        eps: first(&facts, &["DilutedEPS"]),
        shares_outstanding: first(
            &facts,
            &[
                "DilutedAverageShares",
                "OrdinarySharesNumber",
                "BasicAverageShares",
            ],
        ),
        year_end_price: None,
        cash: first(
            &facts,
            &[
                "CashCashEquivalentsAndShortTermInvestments",
                "CashAndCashEquivalents",
            ],
        ),
        debt: first(&facts, &["TotalDebt"]),
        equity: first(&facts, &["StockholdersEquity"]),
        pretax_income: first(&facts, &["PretaxIncome"]),
        tax_expense: first(&facts, &["TaxProvision"]),
        interest_expense: first(&facts, &["InterestExpense", "InterestExpenseNonOperating"])
            .map(f64::abs)
            .filter(|value| *value > 0.0),
        period_end,
    }
}

fn first(facts: &PeriodFacts, names: &[&str]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| facts.fields.get(*name).copied())
}

fn unprefixed(key: &str) -> &str {
    key.strip_prefix("annual")
        .or_else(|| key.strip_prefix("quarterly"))
        .or_else(|| key.strip_prefix("trailing"))
        .unwrap_or(key)
}

fn yahoo_symbol(ticker: &str) -> String {
    ticker.replace('.', "-")
}

fn json_f64(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| value.as_i64().map(|n| n as f64))
}

fn unix_date(ts: i64) -> String {
    let z = ts.div_euclid(86_400) + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if month <= 2 { year + 1 } else { year };
    format!("{year:04}-{month:02}-{day:02}")
}

fn period_label(annual: bool, period_end: &str) -> String {
    if annual {
        return "FY".into();
    }
    match period_end
        .get(5..7)
        .and_then(|month| month.parse::<u8>().ok())
    {
        Some(month) if (1..=3).contains(&month) => "Q1".into(),
        Some(month) if (4..=6).contains(&month) => "Q2".into(),
        Some(month) if (7..=9).contains(&month) => "Q3".into(),
        Some(month) if (10..=12).contains(&month) => "Q4".into(),
        _ => "Q".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_unix_date() {
        assert_eq!(unix_date(0), "1970-01-01");
        assert_eq!(unix_date(1_704_067_200), "2024-01-01");
    }

    #[test]
    fn quarterly_labels_follow_period_end_month() {
        assert_eq!(period_label(true, "2025-09-30"), "FY");
        assert_eq!(period_label(false, "2025-03-31"), "Q1");
        assert_eq!(period_label(false, "2025-06-30"), "Q2");
        assert_eq!(period_label(false, "2025-09-30"), "Q3");
        assert_eq!(period_label(false, "2025-12-31"), "Q4");
        assert_eq!(period_label(false, "bad"), "Q");
    }

    #[test]
    fn class_shares_use_yahoo_hyphen() {
        assert_eq!(yahoo_symbol("BRK.B"), "BRK-B");
        assert_eq!(yahoo_symbol("AAPL"), "AAPL");
    }

    #[test]
    fn timeseries_builds_statements_and_keeps_interest_as_a_cost() {
        let payload = serde_json::json!({
            "timeseries": {
                "result": [
                    {
                        "meta": { "type": ["annualTotalRevenue"] },
                        "annualTotalRevenue": [
                            {
                                "asOfDate": "2024-09-30",
                                "currencyCode": "USD",
                                "reportedValue": { "raw": 100.0 }
                            }
                        ]
                    },
                    {
                        "meta": { "type": ["annualOperatingCashFlow"] },
                        "annualOperatingCashFlow": [
                            {
                                "asOfDate": "2024-09-30",
                                "reportedValue": { "raw": 40.0 }
                            }
                        ]
                    },
                    {
                        "meta": { "type": ["annualCapitalExpenditure"] },
                        "annualCapitalExpenditure": [
                            {
                                "asOfDate": "2024-09-30",
                                "reportedValue": { "raw": -8.0 }
                            }
                        ]
                    },
                    {
                        "meta": { "type": ["annualInterestExpense"] },
                        "annualInterestExpense": [
                            {
                                "asOfDate": "2024-09-30",
                                "reportedValue": { "raw": -2.5 }
                            }
                        ]
                    },
                    {
                        "meta": { "type": ["annualCashCashEquivalentsAndShortTermInvestments"] },
                        "annualCashCashEquivalentsAndShortTermInvestments": [
                            {
                                "asOfDate": "2024-09-30",
                                "reportedValue": { "raw": 12.0 }
                            }
                        ]
                    },
                    {
                        "meta": { "type": ["annualTotalDebt"] },
                        "annualTotalDebt": [
                            {
                                "asOfDate": "2024-09-30",
                                "reportedValue": { "raw": 5.0 }
                            }
                        ]
                    }
                ]
            }
        });
        let rows = rows_from_timeseries(&payload, true);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].revenue, Some(100.0));
        assert_eq!(rows[0].free_cash_flow, Some(32.0));
        assert_eq!(rows[0].interest_expense, Some(2.5));
        assert_eq!(rows[0].cash, Some(12.0));
        assert_eq!(rows[0].debt, Some(5.0));
        assert_eq!(rows[0].fiscal_period, "FY");
    }

    #[test]
    fn prefers_reported_free_cash_flow() {
        let payload = serde_json::json!({
            "timeseries": {
                "result": [
                    {
                        "meta": { "type": ["annualOperatingCashFlow"] },
                        "annualOperatingCashFlow": [
                            { "asOfDate": "2024-09-30", "reportedValue": { "raw": 40.0 } }
                        ]
                    },
                    {
                        "meta": { "type": ["annualCapitalExpenditure"] },
                        "annualCapitalExpenditure": [
                            { "asOfDate": "2024-09-30", "reportedValue": { "raw": -8.0 } }
                        ]
                    },
                    {
                        "meta": { "type": ["annualFreeCashFlow"] },
                        "annualFreeCashFlow": [
                            { "asOfDate": "2024-09-30", "reportedValue": { "raw": 31.0 } }
                        ]
                    }
                ]
            }
        });
        let rows = rows_from_timeseries(&payload, true);
        assert_eq!(rows[0].free_cash_flow, Some(31.0));
    }
}
