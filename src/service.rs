use std::path::PathBuf;
use std::sync::Arc;

use crate::domain::{DcfAssumptions, PeriodKind, newest_first, normalize_ticker};
use crate::error::AppError;
use crate::provider::{FetchCtx, ProviderKind, http_client, resolve_provider};
use crate::research::{
    Multiples, Point, SeriesSet, Snapshot, StatementRow, current_multiples, price_series,
    quarterly_series, series, snapshot, statements,
};
use crate::store::Store;
use crate::valuation::{self, DcfResult, Sensitivity};

pub struct AppState {
    pub store: Arc<Store>,
    pub testdata_dir: PathBuf,
    pub http: reqwest::Client,
}

impl AppState {
    pub fn new(store: Store, testdata_dir: PathBuf) -> Result<Self, AppError> {
        Ok(Self {
            store: Arc::new(store),
            testdata_dir,
            http: http_client()?,
        })
    }

    fn ctx(&self) -> Result<FetchCtx, AppError> {
        Ok(FetchCtx {
            testdata_dir: self.testdata_dir.clone(),
            http: self.http.clone(),
            fmp_key: self.store.fmp_key()?,
        })
    }

    fn provider(&self) -> Result<ProviderKind, AppError> {
        ProviderKind::parse(&self.store.provider()?)
    }

    fn fetch_provider(&self, ticker: &str) -> Result<ProviderKind, AppError> {
        Ok(resolve_provider(
            self.provider()?,
            &self.testdata_dir,
            ticker,
        ))
    }

    pub fn settings(&self) -> Result<SettingsView, AppError> {
        Ok(SettingsView {
            provider: self.store.provider()?,
            providers: ProviderKind::all().to_vec(),
            has_fmp_key: self.store.fmp_key()?.is_some(),
        })
    }

    pub fn update_settings(&self, update: SettingsUpdate) -> Result<SettingsView, AppError> {
        if let Some(provider) = update.provider {
            let kind = ProviderKind::parse(&provider)?;
            self.store.set_provider(kind.as_str())?;
        }
        match update.fmp_key.as_deref() {
            None => {}
            Some("") => self.store.set_fmp_key(None)?,
            Some(key) => self.store.set_fmp_key(Some(key))?,
        }
        self.settings()
    }

    pub fn watchlist(&self) -> Result<Vec<WatchItem>, AppError> {
        let tickers = self.store.watchlist()?;
        let mut items = Vec::new();
        for ticker in tickers {
            items.push(self.watch_item(&ticker)?);
        }
        Ok(items)
    }

    pub async fn add_watch(&self, ticker: &str) -> Result<Vec<WatchItem>, AppError> {
        let ticker = normalize_ticker(ticker).map_err(AppError::BadRequest)?;
        let had_quote = self.store.quote(&ticker)?.is_some();
        self.store.add_watch(&ticker)?;
        if !had_quote {
            if let Err(error) = self.refresh(&ticker).await {
                let _ = self.store.remove_watch(&ticker);
                return Err(error);
            }
        }
        self.watchlist()
    }

    pub fn remove_watch(&self, ticker: &str) -> Result<Vec<WatchItem>, AppError> {
        let ticker = normalize_ticker(ticker).map_err(AppError::BadRequest)?;
        if !self.store.remove_watch(&ticker)? {
            return Err(AppError::NotFound(format!(
                "{ticker} is not on the watchlist"
            )));
        }
        self.watchlist()
    }

    pub async fn company(&self, ticker: &str, force: bool) -> Result<CompanyView, AppError> {
        let ticker = normalize_ticker(ticker).map_err(AppError::BadRequest)?;
        // Cache wins until Refresh. Changing the active provider does not refetch names.
        let missing = self.store.quote(&ticker)?.is_none()
            || self
                .store
                .financials(&ticker, PeriodKind::Annual)?
                .is_none()
            || self.store.prices(&ticker)?.is_none();
        if force || missing {
            self.refresh(&ticker).await?;
        }
        self.company_from_cache(&ticker)
    }

    pub fn save_dcf(
        &self,
        ticker: &str,
        assumptions: DcfAssumptions,
    ) -> Result<CompanyView, AppError> {
        let ticker = normalize_ticker(ticker).map_err(AppError::BadRequest)?;
        if !(0.0..=1.0).contains(&assumptions.growth) {
            return Err(AppError::BadRequest(
                "growth must be between 0 and 1".into(),
            ));
        }
        if !(0.01..=1.0).contains(&assumptions.desired_return) {
            return Err(AppError::BadRequest(
                "desired return must be between 0.01 and 1".into(),
            ));
        }
        if assumptions.desired_return <= assumptions.growth {
            return Err(AppError::BadRequest(
                "desired return must be greater than growth".into(),
            ));
        }
        self.store.put_dcf(&ticker, &assumptions)?;
        self.company_from_cache(&ticker)
    }

    pub fn save_note(&self, ticker: &str, body: &str) -> Result<CompanyView, AppError> {
        let ticker = normalize_ticker(ticker).map_err(AppError::BadRequest)?;
        let body = body.trim();
        if body.chars().count() > 4000 {
            return Err(AppError::BadRequest(
                "note must be at most 4000 characters".into(),
            ));
        }
        self.store.put_note(&ticker, body)?;
        self.company_from_cache(&ticker)
    }

    async fn refresh(&self, ticker: &str) -> Result<(), AppError> {
        let provider = self.fetch_provider(ticker)?;
        let ctx = self.ctx()?;
        let mut quote = provider.quote(&ctx, ticker).await?;
        let annual = provider.financials(&ctx, ticker, true).await?;
        if quote.market_cap.is_none() {
            if let Some(shares) = newest_first(&annual)
                .iter()
                .find_map(|row| row.shares_outstanding.filter(|shares| *shares > 0.0))
            {
                quote.shares_outstanding = Some(shares);
                if quote.price > 0.0 {
                    quote.market_cap = Some(quote.price * shares);
                }
            }
        }
        // Quarterly is optional. A flake keeps the last same-provider quarters instead of failing the refresh.
        let quarterly = provider.financials(&ctx, ticker, false).await.ok();
        let prices = provider.prices(&ctx, ticker).await?;
        self.store.put_refresh(
            ticker,
            &quote,
            &annual,
            quarterly.as_deref(),
            &prices,
            provider.as_str(),
        )?;
        Ok(())
    }

    fn company_from_cache(&self, ticker: &str) -> Result<CompanyView, AppError> {
        let quote = self
            .store
            .quote(ticker)?
            .ok_or_else(|| AppError::NotFound(format!("no cached quote for {ticker}")))?;
        let annual = self
            .store
            .financials(ticker, PeriodKind::Annual)?
            .filter(|row| row.provider == quote.provider)
            .map(|row| row.value)
            .unwrap_or_default();
        let quarterly = self
            .store
            .financials(ticker, PeriodKind::Quarterly)?
            .filter(|row| row.provider == quote.provider)
            .map(|row| row.value)
            .unwrap_or_default();
        let prices = self
            .store
            .prices(ticker)?
            .filter(|row| row.provider == quote.provider)
            .map(|row| row.value)
            .unwrap_or_default();
        let assumptions = self.store.dcf(ticker)?;
        let note = self.store.note(ticker)?;
        let multiples = current_multiples(
            quote.value.price,
            quote.value.market_cap,
            &annual,
            &quarterly,
        );
        let snap = snapshot(&annual, &quarterly, &prices, &multiples);
        let fcf_yield_vs_hurdle = vs_hurdle(multiples.fcf_yield, assumptions.desired_return);
        let fcf_yield_ev_vs_hurdle = vs_hurdle(multiples.fcf_yield_ev, assumptions.desired_return);
        let fcf_power_vs_hurdle = vs_hurdle(snap.fcf_power, assumptions.desired_return);
        let ocf_power_vs_hurdle = vs_hurdle(snap.ocf_power, assumptions.desired_return);
        let seed = valuation::ttm_seed(&annual, &quarterly);
        let net_cash_per_share = cash_per_share(
            multiples.net_cash,
            valuation::ttm_share_count(&annual, &quarterly).or(quote
                .value
                .shares_outstanding
                .filter(|shares| *shares > 0.0)),
        );
        let dcf = seed.map(|(seed, kind)| {
            valuation::project(
                &assumptions,
                seed,
                kind,
                quote.value.price,
                net_cash_per_share,
            )
        });
        let sensitivity = seed.map(|(seed, kind)| {
            valuation::sensitivity(
                seed,
                kind,
                quote.value.price,
                &assumptions,
                net_cash_per_share,
            )
        });
        Ok(CompanyView {
            ticker: quote.value.ticker.clone(),
            name: quote.value.name.clone(),
            sector: quote.value.sector.clone(),
            currency: quote.value.currency.clone(),
            price: quote.value.price,
            market_cap: quote.value.market_cap,
            provider: quote.provider,
            active_provider: self.fetch_provider(ticker)?.as_str().to_string(),
            fetched_at: quote.fetched_at,
            multiples,
            snapshot: snap,
            annual: statements(&annual),
            quarterly: statements(&quarterly),
            series: series(&annual, &prices),
            quarterly_series: quarterly_series(&quarterly, &annual, &prices),
            price_series: price_series(&prices, &annual),
            dcf,
            sensitivity,
            assumptions,
            fcf_yield_vs_hurdle,
            fcf_yield_ev_vs_hurdle,
            fcf_power_vs_hurdle,
            ocf_power_vs_hurdle,
            note,
        })
    }

    fn watch_item(&self, ticker: &str) -> Result<WatchItem, AppError> {
        let quote = self.store.quote(ticker)?;
        let provider = quote.as_ref().map(|row| row.provider.as_str());
        let annual = self
            .store
            .financials(ticker, PeriodKind::Annual)?
            .filter(|row| provider == Some(row.provider.as_str()))
            .map(|row| row.value)
            .unwrap_or_default();
        let quarterly = self
            .store
            .financials(ticker, PeriodKind::Quarterly)?
            .filter(|row| provider == Some(row.provider.as_str()))
            .map(|row| row.value)
            .unwrap_or_default();
        let prices = self
            .store
            .prices(ticker)?
            .filter(|row| provider == Some(row.provider.as_str()))
            .map(|row| row.value)
            .unwrap_or_default();
        let multiples = quote.as_ref().map(|row| {
            current_multiples(row.value.price, row.value.market_cap, &annual, &quarterly)
        });
        let snap = multiples
            .as_ref()
            .map(|row| snapshot(&annual, &quarterly, &prices, row));
        let assumptions = self.store.dcf(ticker)?;
        let net_cash_per_share = cash_per_share(
            multiples.as_ref().and_then(|row| row.net_cash),
            valuation::ttm_share_count(&annual, &quarterly).or(quote
                .as_ref()
                .and_then(|row| row.value.shares_outstanding)
                .filter(|shares| *shares > 0.0)),
        );
        let upside = quote
            .as_ref()
            .and_then(|row| {
                valuation::ttm_seed(&annual, &quarterly).map(|(seed, kind)| {
                    valuation::project(
                        &assumptions,
                        seed,
                        kind,
                        row.value.price,
                        net_cash_per_share,
                    )
                    .upside
                })
            })
            .flatten();
        Ok(WatchItem {
            ticker: ticker.to_string(),
            name: quote.as_ref().map(|row| row.value.name.clone()),
            provider: quote.as_ref().map(|row| row.provider.clone()),
            price: quote.as_ref().map(|row| row.value.price),
            currency: quote.as_ref().map(|row| row.value.currency.clone()),
            pe: multiples.as_ref().and_then(|row| row.pe),
            p_fcf: multiples.as_ref().and_then(|row| row.p_fcf),
            p_ocf: multiples.as_ref().and_then(|row| row.p_ocf),
            pe_vs_median: snap.as_ref().and_then(|row| row.pe_vs_median),
            p_fcf_vs_median: snap.as_ref().and_then(|row| row.p_fcf_vs_median),
            p_ocf_vs_median: snap.as_ref().and_then(|row| row.p_ocf_vs_median),
            pe_percentile: snap.as_ref().and_then(|row| row.pe_percentile),
            revenue_cagr: snap.as_ref().and_then(|row| row.revenue_cagr),
            revenue_cagr_5y: snap.as_ref().and_then(|row| row.revenue_cagr_5y),
            revenue_cagr_fade: snap.as_ref().and_then(|row| row.revenue_cagr_fade),
            fcf_ps_cagr: snap.as_ref().and_then(|row| row.fcf_ps_cagr),
            fcf_yield: multiples.as_ref().and_then(|row| row.fcf_yield),
            fcf_yield_ev: multiples.as_ref().and_then(|row| row.fcf_yield_ev),
            interest_coverage: snap.as_ref().and_then(|row| row.interest_coverage),
            net_cash: multiples.as_ref().and_then(|row| row.net_cash),
            fcf_yield_vs_median: snap.as_ref().and_then(|row| row.fcf_yield_vs_median),
            fcf_conversion: snap.as_ref().and_then(|row| row.fcf_conversion),
            fcf_yield_vs_hurdle: multiples
                .as_ref()
                .and_then(|row| vs_hurdle(row.fcf_yield, assumptions.desired_return)),
            fcf_power_vs_hurdle: snap
                .as_ref()
                .and_then(|row| vs_hurdle(row.fcf_power, assumptions.desired_return)),
            ocf_power_vs_hurdle: snap
                .as_ref()
                .and_then(|row| vs_hurdle(row.ocf_power, assumptions.desired_return)),
            years_to_median_p_fcf: snap.as_ref().and_then(|row| row.years_to_median_p_fcf),
            years_to_median_pe: snap.as_ref().and_then(|row| row.years_to_median_pe),
            years_to_median_p_ocf: snap.as_ref().and_then(|row| row.years_to_median_p_ocf),
            upside,
            note: excerpt(&self.store.note(ticker)?),
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SettingsView {
    pub provider: String,
    pub providers: Vec<&'static str>,
    pub has_fmp_key: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct SettingsUpdate {
    pub provider: Option<String>,
    pub fmp_key: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct WatchItem {
    pub ticker: String,
    pub name: Option<String>,
    pub provider: Option<String>,
    pub price: Option<f64>,
    pub currency: Option<String>,
    pub pe: Option<f64>,
    pub p_fcf: Option<f64>,
    pub p_ocf: Option<f64>,
    pub pe_vs_median: Option<f64>,
    pub p_fcf_vs_median: Option<f64>,
    pub p_ocf_vs_median: Option<f64>,
    pub pe_percentile: Option<f64>,
    pub revenue_cagr: Option<f64>,
    pub revenue_cagr_5y: Option<f64>,
    pub revenue_cagr_fade: Option<f64>,
    pub fcf_ps_cagr: Option<f64>,
    pub fcf_yield: Option<f64>,
    pub fcf_yield_ev: Option<f64>,
    pub interest_coverage: Option<f64>,
    pub net_cash: Option<f64>,
    pub fcf_yield_vs_median: Option<f64>,
    pub fcf_conversion: Option<f64>,
    pub fcf_yield_vs_hurdle: Option<f64>,
    pub fcf_power_vs_hurdle: Option<f64>,
    pub ocf_power_vs_hurdle: Option<f64>,
    pub years_to_median_p_fcf: Option<f64>,
    pub years_to_median_pe: Option<f64>,
    pub years_to_median_p_ocf: Option<f64>,
    pub upside: Option<f64>,
    pub note: String,
}

fn excerpt(note: &str) -> String {
    let trimmed = note.trim();
    if trimmed.chars().count() <= 80 {
        return trimmed.to_string();
    }
    let cut: String = trimmed.chars().take(77).collect();
    format!("{cut}…")
}

#[derive(Debug, serde::Serialize)]
pub struct CompanyView {
    pub ticker: String,
    pub name: String,
    pub sector: String,
    pub currency: String,
    pub price: f64,
    pub market_cap: Option<f64>,
    pub provider: String,
    pub active_provider: String,
    pub fetched_at: String,
    pub multiples: Multiples,
    pub snapshot: Snapshot,
    pub annual: Vec<StatementRow>,
    pub quarterly: Vec<StatementRow>,
    pub series: SeriesSet,
    pub quarterly_series: SeriesSet,
    pub price_series: Vec<Point>,
    pub dcf: Option<DcfResult>,
    pub sensitivity: Option<Sensitivity>,
    pub assumptions: DcfAssumptions,
    pub fcf_yield_vs_hurdle: Option<f64>,
    pub fcf_yield_ev_vs_hurdle: Option<f64>,
    pub fcf_power_vs_hurdle: Option<f64>,
    pub ocf_power_vs_hurdle: Option<f64>,
    pub note: String,
}

fn vs_hurdle(yield_: Option<f64>, hurdle: f64) -> Option<f64> {
    yield_.map(|yield_| yield_ - hurdle)
}

fn cash_per_share(net_cash: Option<f64>, shares: Option<f64>) -> Option<f64> {
    match (net_cash, shares) {
        (Some(cash), Some(shares)) if shares > 0.0 => Some(cash / shares),
        _ => None,
    }
}
