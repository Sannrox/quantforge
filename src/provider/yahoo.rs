use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;

use crate::domain::{Financials, Ohlcv, Quote};
use crate::error::AppError;

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
    #[serde(default)]
    symbol: Option<String>,
    #[serde(default, rename = "shortName")]
    short_name: Option<String>,
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

pub async fn quote(http: &Client, ticker: &str) -> Result<Quote, AppError> {
    let summary = quote_summary(http, ticker).await.ok();
    let chart = chart(http, ticker, "1y").await?;
    let result = chart_result(&chart)?;
    let price = result
        .meta
        .regular_market_price
        .or(result.meta.chart_previous_close)
        .ok_or_else(|| AppError::Provider(format!("yahoo: no price for {ticker}")))?;
    let profile = summary
        .as_ref()
        .and_then(|value| value.pointer("/quoteSummary/result/0"));
    let name = profile
        .and_then(|value| value.pointer("/price/longName").and_then(json_string))
        .or_else(|| {
            profile.and_then(|value| value.pointer("/price/shortName").and_then(json_string))
        })
        .or_else(|| result.meta.short_name.clone())
        .unwrap_or_else(|| ticker.to_string());
    let sector = profile
        .and_then(|value| value.pointer("/assetProfile/sector").and_then(json_string))
        .unwrap_or_default();
    let shares = profile.and_then(|value| {
        value
            .pointer("/defaultKeyStatistics/sharesOutstanding/raw")
            .and_then(json_f64)
    });
    let market_cap =
        profile.and_then(|value| value.pointer("/price/marketCap/raw").and_then(json_f64));
    Ok(Quote {
        ticker: result
            .meta
            .symbol
            .clone()
            .unwrap_or_else(|| ticker.to_string()),
        name,
        sector,
        price,
        currency: result.meta.currency.clone().unwrap_or_else(|| "USD".into()),
        market_cap,
        shares_outstanding: shares,
    })
}

pub async fn financials(
    http: &Client,
    ticker: &str,
    annual: bool,
) -> Result<Vec<Financials>, AppError> {
    let summary = quote_summary(http, ticker).await?;
    let root = summary
        .pointer("/quoteSummary/result/0")
        .ok_or_else(|| AppError::Provider(format!("yahoo: no quoteSummary for {ticker}")))?;
    let income_key = if annual {
        "incomeStatementHistory"
    } else {
        "incomeStatementHistoryQuarterly"
    };
    let cash_key = if annual {
        "cashflowStatementHistory"
    } else {
        "cashflowStatementHistoryQuarterly"
    };
    let balance_key = if annual {
        "balanceSheetHistory"
    } else {
        "balanceSheetHistoryQuarterly"
    };
    let income = statements(root, income_key, "incomeStatementHistory");
    let cash = statements(root, cash_key, "cashflowStatements");
    let balance = statements(root, balance_key, "balanceSheetStatements");
    if income.is_empty() {
        return Err(AppError::Provider(format!(
            "yahoo: no {} statements for {ticker}",
            if annual { "annual" } else { "quarterly" }
        )));
    }
    let mut rows = Vec::new();
    for item in income {
        let end = item
            .get("endDate")
            .and_then(|value| value.get("fmt"))
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        let cash_row = find_by_end(&cash, &end);
        let balance_row = find_by_end(&balance, &end);
        let revenue = raw(&item, "totalRevenue");
        let net_income = raw(&item, "netIncome");
        // Share count only. Income/balance `commonStock` and cash `commonStockIssuance` are dollars.
        let shares = raw(&item, "dilutedAverageShares");
        let operating_cf = cash_row.and_then(|row| raw(row, "totalCashFromOperatingActivities"));
        let capex = cash_row.and_then(|row| raw(row, "capitalExpenditures"));
        let fcf = match (operating_cf, capex) {
            (Some(ocf), Some(capex)) => Some(ocf + capex),
            (Some(ocf), None) => Some(ocf),
            _ => None,
        };
        let eps = raw(&item, "dilutedEPS").or_else(|| match (net_income, shares) {
            (Some(ni), Some(shares)) if shares.abs() > 0.0 => Some(ni / shares),
            _ => None,
        });
        let fiscal_period = period_label(annual, &end);
        rows.push(Financials {
            period_end: end,
            fiscal_period,
            currency: "USD".into(), // v1: no FX; Yahoo statement currency is not converted
            revenue,
            ebitda: raw(&item, "ebitda").or_else(|| raw(&item, "normalizedEBITDA")),
            gross_profit: raw(&item, "grossProfit"),
            operating_income: raw(&item, "operatingIncome"),
            net_income,
            operating_cash_flow: operating_cf,
            free_cash_flow: fcf,
            eps,
            shares_outstanding: shares,
            year_end_price: None,
            cash: balance_row.and_then(cash_like),
            debt: balance_row.and_then(interest_bearing_debt),
            equity: balance_row.and_then(book_equity),
            pretax_income: raw(&item, "incomeBeforeTax"),
            tax_expense: raw(&item, "incomeTaxExpense"),
        });
    }
    Ok(rows)
}

pub async fn prices(http: &Client, ticker: &str) -> Result<Vec<Ohlcv>, AppError> {
    let chart = chart(http, ticker, "max").await?;
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
    let response = http.get(url).send().await?;
    if !response.status().is_success() {
        return Err(AppError::Provider(format!(
            "yahoo chart {}: {}",
            ticker,
            response.status()
        )));
    }
    Ok(response.json().await?)
}

async fn quote_summary(http: &Client, ticker: &str) -> Result<Value, AppError> {
    let url = format!(
        "https://query2.finance.yahoo.com/v10/finance/quoteSummary/{ticker}?modules=assetProfile,price,defaultKeyStatistics,financialData,incomeStatementHistory,incomeStatementHistoryQuarterly,balanceSheetHistory,balanceSheetHistoryQuarterly,cashflowStatementHistory,cashflowStatementHistoryQuarterly"
    );
    let response = http.get(url).send().await?;
    if !response.status().is_success() {
        return Err(AppError::Provider(format!(
            "yahoo quoteSummary {}: {}",
            ticker,
            response.status()
        )));
    }
    Ok(response.json().await?)
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

fn statements<'a>(root: &'a Value, module: &str, list_key: &str) -> Vec<&'a Value> {
    let Some(module) = root.get(module) else {
        return Vec::new();
    };
    for key in [
        list_key,
        "incomeStatementHistory",
        "cashflowStatements",
        "balanceSheetStatements",
    ] {
        if let Some(rows) = module.get(key).and_then(Value::as_array) {
            return rows.iter().collect();
        }
    }
    Vec::new()
}

fn find_by_end<'a>(rows: &[&'a Value], end: &str) -> Option<&'a Value> {
    rows.iter().copied().find(|row| {
        row.get("endDate")
            .and_then(|value| value.get("fmt"))
            .and_then(Value::as_str)
            == Some(end)
    })
}

fn raw(row: &Value, field: &str) -> Option<f64> {
    row.get(field)
        .and_then(|value| value.get("raw"))
        .and_then(json_f64)
}

fn cash_like(row: &Value) -> Option<f64> {
    raw(row, "cashAndShortTermInvestments")
        .or_else(|| sum_opt(raw(row, "cash"), raw(row, "shortTermInvestments")))
        .or_else(|| raw(row, "cash"))
}

fn interest_bearing_debt(row: &Value) -> Option<f64> {
    raw(row, "totalDebt")
        .or_else(|| sum_opt(raw(row, "shortLongTermDebt"), raw(row, "longTermDebt")))
        .or_else(|| raw(row, "longTermDebt"))
}

fn book_equity(row: &Value) -> Option<f64> {
    raw(row, "totalStockholderEquity")
        .or_else(|| raw(row, "totalStockholdersEquity"))
        .or_else(|| raw(row, "stockholdersEquity"))
}

fn sum_opt(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        (Some(left), None) => Some(left),
        (None, Some(right)) => Some(right),
        (None, None) => None,
    }
}

fn json_f64(value: &Value) -> Option<f64> {
    value.as_f64().or_else(|| value.as_i64().map(|n| n as f64))
}

fn json_string(value: &Value) -> Option<String> {
    value.as_str().map(str::to_string)
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
    fn reads_cash_debt_and_equity_aliases() {
        let row = serde_json::json!({
            "cash": { "raw": 10.0 },
            "shortTermInvestments": { "raw": 4.0 },
            "shortLongTermDebt": { "raw": 3.0 },
            "longTermDebt": { "raw": 8.0 },
            "totalStockholderEquity": { "raw": 40.0 }
        });
        assert_eq!(cash_like(&row), Some(14.0));
        assert_eq!(interest_bearing_debt(&row), Some(11.0));
        assert_eq!(book_equity(&row), Some(40.0));
    }
}
