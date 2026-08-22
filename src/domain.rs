use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quote {
    pub ticker: String,
    pub name: String,
    pub sector: String,
    pub price: f64,
    pub currency: String,
    #[serde(default)]
    pub market_cap: Option<f64>,
    #[serde(default)]
    pub shares_outstanding: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Financials {
    pub period_end: String,
    pub fiscal_period: String,
    pub currency: String,
    #[serde(default)]
    pub revenue: Option<f64>,
    #[serde(default)]
    pub ebitda: Option<f64>,
    #[serde(default)]
    pub gross_profit: Option<f64>,
    #[serde(default)]
    pub operating_income: Option<f64>,
    #[serde(default)]
    pub net_income: Option<f64>,
    #[serde(default)]
    pub operating_cash_flow: Option<f64>,
    #[serde(default)]
    pub free_cash_flow: Option<f64>,
    #[serde(default)]
    pub eps: Option<f64>,
    #[serde(default)]
    pub shares_outstanding: Option<f64>,
    #[serde(default)]
    pub year_end_price: Option<f64>,
    #[serde(default)]
    pub cash: Option<f64>,
    #[serde(default)]
    pub debt: Option<f64>,
    #[serde(default)]
    pub equity: Option<f64>,
    #[serde(default)]
    pub pretax_income: Option<f64>,
    #[serde(default)]
    pub tax_expense: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ohlcv {
    pub date: String,
    pub close: f64,
    #[serde(default)]
    pub adj_close: Option<f64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PeriodKind {
    Annual,
    Quarterly,
}

impl PeriodKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Annual => "annual",
            Self::Quarterly => "quarterly",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "annual" => Some(Self::Annual),
            "quarterly" => Some(Self::Quarterly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DcfAssumptions {
    pub growth: f64,
    pub desired_return: f64,
}

impl Default for DcfAssumptions {
    fn default() -> Self {
        Self {
            growth: 0.08,
            desired_return: 0.12,
        }
    }
}

pub fn normalize_ticker(ticker: &str) -> Result<String, String> {
    let ticker = ticker.trim().to_ascii_uppercase();
    if ticker.is_empty()
        || ticker.len() > 16
        || ticker.chars().all(|c| c == '.' || c == '-')
        || !ticker
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-')
    {
        return Err("ticker must be 1–16 letters, digits, '.', or '-'".into());
    }
    Ok(ticker)
}

pub fn newest_first(rows: &[Financials]) -> Vec<Financials> {
    let mut rows = rows.to_vec();
    rows.sort_by(|left, right| right.period_end.cmp(&left.period_end));
    rows
}

#[cfg(test)]
mod tests {
    use super::normalize_ticker;

    #[test]
    fn rejects_path_segment_tickers() {
        assert!(normalize_ticker(".").is_err());
        assert!(normalize_ticker("..").is_err());
        assert!(normalize_ticker("...").is_err());
        assert!(normalize_ticker("-").is_err());
        assert!(normalize_ticker("--").is_err());
        assert!(normalize_ticker(".-").is_err());
        assert_eq!(normalize_ticker("brk.b").unwrap(), "BRK.B");
    }
}
