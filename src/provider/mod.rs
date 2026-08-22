use std::path::PathBuf;
use std::time::Duration;

use reqwest::Client;

use crate::domain::{Financials, Ohlcv, Quote};
use crate::error::AppError;

mod fixture;
mod fmp;
mod yahoo;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderKind {
    Fixture,
    Yahoo,
    Fmp,
}

impl ProviderKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fixture => "fixture",
            Self::Yahoo => "yahoo",
            Self::Fmp => "fmp",
        }
    }

    pub fn parse(value: &str) -> Result<Self, AppError> {
        match value {
            "fixture" => Ok(Self::Fixture),
            "yahoo" => Ok(Self::Yahoo),
            "fmp" => Ok(Self::Fmp),
            other => Err(AppError::BadRequest(format!(
                "unknown provider '{other}'; use fixture, yahoo, or fmp"
            ))),
        }
    }

    pub fn all() -> &'static [&'static str] {
        &["fixture", "yahoo", "fmp"]
    }
}

pub struct FetchCtx {
    pub testdata_dir: PathBuf,
    pub http: Client,
    pub fmp_key: Option<String>,
}

impl ProviderKind {
    pub async fn quote(&self, ctx: &FetchCtx, ticker: &str) -> Result<Quote, AppError> {
        match self {
            Self::Fixture => fixture::quote(&ctx.testdata_dir, ticker),
            Self::Yahoo => yahoo::quote(&ctx.http, ticker).await,
            Self::Fmp => fmp::quote(&ctx.http, ctx.fmp_key.as_deref(), ticker).await,
        }
    }

    pub async fn financials(
        &self,
        ctx: &FetchCtx,
        ticker: &str,
        annual: bool,
    ) -> Result<Vec<Financials>, AppError> {
        match self {
            Self::Fixture => fixture::financials(&ctx.testdata_dir, ticker, annual),
            Self::Yahoo => yahoo::financials(&ctx.http, ticker, annual).await,
            Self::Fmp => fmp::financials(&ctx.http, ctx.fmp_key.as_deref(), ticker, annual).await,
        }
    }

    pub async fn prices(&self, ctx: &FetchCtx, ticker: &str) -> Result<Vec<Ohlcv>, AppError> {
        match self {
            Self::Fixture => fixture::prices(&ctx.testdata_dir, ticker),
            Self::Yahoo => yahoo::prices(&ctx.http, ticker).await,
            Self::Fmp => fmp::prices(&ctx.http, ctx.fmp_key.as_deref(), ticker).await,
        }
    }
}

pub fn http_client() -> Result<Client, AppError> {
    Client::builder()
        .user_agent("QuantForge/0.1 (+https://github.com/Sannrox/quantforge)")
        .connect_timeout(Duration::from_secs(10))
        .timeout(Duration::from_secs(30))
        .build()
        .map_err(|error| AppError::Message(format!("http client: {error}")))
}
