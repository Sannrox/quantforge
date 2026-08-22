use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::domain::{DcfAssumptions, Financials, Ohlcv, PeriodKind, Quote};
use crate::error::AppError;

pub const DEFAULT_PROVIDER: &str = "fixture";

pub struct Store {
    conn: Mutex<Connection>,
}

#[derive(Debug, Clone)]
pub struct Cached<T> {
    pub value: T,
    pub provider: String,
    pub fetched_at: String,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, AppError> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|error| AppError::Message(format!("create db dir: {error}")))?;
        }
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "
            PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS watchlist (
                ticker TEXT PRIMARY KEY,
                added_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS quotes (
                ticker TEXT PRIMARY KEY,
                payload TEXT NOT NULL,
                fetched_at TEXT NOT NULL,
                provider TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS financials (
                ticker TEXT NOT NULL,
                period_kind TEXT NOT NULL,
                payload TEXT NOT NULL,
                fetched_at TEXT NOT NULL,
                provider TEXT NOT NULL,
                PRIMARY KEY (ticker, period_kind)
            );
            CREATE TABLE IF NOT EXISTS prices (
                ticker TEXT PRIMARY KEY,
                payload TEXT NOT NULL,
                fetched_at TEXT NOT NULL,
                provider TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS dcf_assumptions (
                ticker TEXT PRIMARY KEY,
                growth REAL NOT NULL,
                desired_return REAL NOT NULL,
                updated_at TEXT NOT NULL
            );
            CREATE TABLE IF NOT EXISTS notes (
                ticker TEXT PRIMARY KEY,
                body TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );
            ",
        )?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn default_path() -> PathBuf {
        home_dir().join(".quantforge").join("quantforge.db")
    }

    pub fn provider(&self) -> Result<String, AppError> {
        Ok(self
            .get_config("data_provider")?
            .unwrap_or_else(|| DEFAULT_PROVIDER.to_string()))
    }

    pub fn set_provider(&self, provider: &str) -> Result<(), AppError> {
        self.set_config("data_provider", provider)
    }

    pub fn fmp_key(&self) -> Result<Option<String>, AppError> {
        Ok(self
            .get_config("fmp_api_key")?
            .filter(|key| !key.is_empty()))
    }

    pub fn set_fmp_key(&self, key: Option<&str>) -> Result<(), AppError> {
        match key {
            None => {
                let conn = self.conn.lock().expect("store lock");
                conn.execute("DELETE FROM config WHERE key = 'fmp_api_key'", [])?;
                Ok(())
            }
            Some(key) => self.set_config("fmp_api_key", key),
        }
    }

    pub fn watchlist(&self) -> Result<Vec<String>, AppError> {
        let conn = self.conn.lock().expect("store lock");
        let mut stmt =
            conn.prepare("SELECT ticker FROM watchlist ORDER BY added_at ASC, ticker ASC")?;
        let rows = stmt.query_map([], |row| row.get(0))?;
        rows.collect::<Result<Vec<_>, _>>().map_err(AppError::from)
    }

    pub fn add_watch(&self, ticker: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("store lock");
        conn.execute(
            "INSERT OR IGNORE INTO watchlist (ticker, added_at) VALUES (?1, datetime('now'))",
            params![ticker],
        )?;
        Ok(())
    }

    pub fn remove_watch(&self, ticker: &str) -> Result<bool, AppError> {
        let conn = self.conn.lock().expect("store lock");
        let changed = conn.execute("DELETE FROM watchlist WHERE ticker = ?1", params![ticker])?;
        Ok(changed > 0)
    }

    pub fn quote(&self, ticker: &str) -> Result<Option<Cached<Quote>>, AppError> {
        self.get_cached(
            "SELECT payload, provider, fetched_at FROM quotes WHERE ticker = ?1",
            ticker,
        )
    }

    pub fn put_refresh(
        &self,
        ticker: &str,
        quote: &Quote,
        annual: &[Financials],
        quarterly: Option<&[Financials]>,
        prices: &[Ohlcv],
        provider: &str,
    ) -> Result<(), AppError> {
        let quote_payload = serde_json::to_string(quote)?;
        let annual_payload = serde_json::to_string(annual)?;
        let quarterly_payload = quarterly.map(serde_json::to_string).transpose()?;
        let prices_payload = serde_json::to_string(prices)?;
        let conn = self.conn.lock().expect("store lock");
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO quotes (ticker, payload, fetched_at, provider) VALUES (?1, ?2, datetime('now'), ?3)
             ON CONFLICT(ticker) DO UPDATE SET payload = excluded.payload, fetched_at = excluded.fetched_at, provider = excluded.provider",
            params![ticker, quote_payload, provider],
        )?;
        tx.execute(
            "INSERT INTO financials (ticker, period_kind, payload, fetched_at, provider)
             VALUES (?1, ?2, ?3, datetime('now'), ?4)
             ON CONFLICT(ticker, period_kind) DO UPDATE SET
                payload = excluded.payload,
                fetched_at = excluded.fetched_at,
                provider = excluded.provider",
            params![
                ticker,
                PeriodKind::Annual.as_str(),
                annual_payload,
                provider
            ],
        )?;
        if let Some(payload) = quarterly_payload.as_ref() {
            tx.execute(
                "INSERT INTO financials (ticker, period_kind, payload, fetched_at, provider)
                 VALUES (?1, ?2, ?3, datetime('now'), ?4)
                 ON CONFLICT(ticker, period_kind) DO UPDATE SET
                    payload = excluded.payload,
                    fetched_at = excluded.fetched_at,
                    provider = excluded.provider",
                params![ticker, PeriodKind::Quarterly.as_str(), payload, provider],
            )?;
        }
        tx.execute(
            "INSERT INTO prices (ticker, payload, fetched_at, provider) VALUES (?1, ?2, datetime('now'), ?3)
             ON CONFLICT(ticker) DO UPDATE SET payload = excluded.payload, fetched_at = excluded.fetched_at, provider = excluded.provider",
            params![ticker, prices_payload, provider],
        )?;
        tx.commit()?;
        Ok(())
    }

    pub fn put_quote(&self, ticker: &str, quote: &Quote, provider: &str) -> Result<(), AppError> {
        self.put_row(
            "INSERT INTO quotes (ticker, payload, fetched_at, provider) VALUES (?1, ?2, datetime('now'), ?3)
             ON CONFLICT(ticker) DO UPDATE SET payload = excluded.payload, fetched_at = excluded.fetched_at, provider = excluded.provider",
            ticker,
            quote,
            provider,
        )
    }

    pub fn financials(
        &self,
        ticker: &str,
        kind: PeriodKind,
    ) -> Result<Option<Cached<Vec<Financials>>>, AppError> {
        let conn = self.conn.lock().expect("store lock");
        let row = conn
            .query_row(
                "SELECT payload, provider, fetched_at FROM financials WHERE ticker = ?1 AND period_kind = ?2",
                params![ticker, kind.as_str()],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, String>(2)?,
                    ))
                },
            )
            .optional()?;
        match row {
            None => Ok(None),
            Some((payload, provider, fetched_at)) => Ok(Some(Cached {
                value: serde_json::from_str(&payload)?,
                provider,
                fetched_at,
            })),
        }
    }

    pub fn put_financials(
        &self,
        ticker: &str,
        kind: PeriodKind,
        rows: &[Financials],
        provider: &str,
    ) -> Result<(), AppError> {
        let payload = serde_json::to_string(rows)?;
        let conn = self.conn.lock().expect("store lock");
        conn.execute(
            "INSERT INTO financials (ticker, period_kind, payload, fetched_at, provider)
             VALUES (?1, ?2, ?3, datetime('now'), ?4)
             ON CONFLICT(ticker, period_kind) DO UPDATE SET
                payload = excluded.payload,
                fetched_at = excluded.fetched_at,
                provider = excluded.provider",
            params![ticker, kind.as_str(), payload, provider],
        )?;
        Ok(())
    }

    pub fn prices(&self, ticker: &str) -> Result<Option<Cached<Vec<Ohlcv>>>, AppError> {
        self.get_cached(
            "SELECT payload, provider, fetched_at FROM prices WHERE ticker = ?1",
            ticker,
        )
    }

    pub fn put_prices(&self, ticker: &str, rows: &[Ohlcv], provider: &str) -> Result<(), AppError> {
        self.put_row(
            "INSERT INTO prices (ticker, payload, fetched_at, provider) VALUES (?1, ?2, datetime('now'), ?3)
             ON CONFLICT(ticker) DO UPDATE SET payload = excluded.payload, fetched_at = excluded.fetched_at, provider = excluded.provider",
            ticker,
            &rows.to_vec(),
            provider,
        )
    }

    pub fn dcf(&self, ticker: &str) -> Result<DcfAssumptions, AppError> {
        let conn = self.conn.lock().expect("store lock");
        let row = conn
            .query_row(
                "SELECT growth, desired_return FROM dcf_assumptions WHERE ticker = ?1",
                params![ticker],
                |row| Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?)),
            )
            .optional()?;
        Ok(match row {
            Some((growth, desired_return)) => DcfAssumptions {
                growth,
                desired_return,
            },
            None => DcfAssumptions::default(),
        })
    }

    pub fn note(&self, ticker: &str) -> Result<String, AppError> {
        let conn = self.conn.lock().expect("store lock");
        Ok(conn
            .query_row(
                "SELECT body FROM notes WHERE ticker = ?1",
                params![ticker],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or_default())
    }

    pub fn put_note(&self, ticker: &str, body: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("store lock");
        if body.is_empty() {
            conn.execute("DELETE FROM notes WHERE ticker = ?1", params![ticker])?;
            return Ok(());
        }
        conn.execute(
            "INSERT INTO notes (ticker, body, updated_at)
             VALUES (?1, ?2, datetime('now'))
             ON CONFLICT(ticker) DO UPDATE SET
                body = excluded.body,
                updated_at = excluded.updated_at",
            params![ticker, body],
        )?;
        Ok(())
    }

    pub fn put_dcf(&self, ticker: &str, assumptions: &DcfAssumptions) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("store lock");
        conn.execute(
            "INSERT INTO dcf_assumptions (ticker, growth, desired_return, updated_at)
             VALUES (?1, ?2, ?3, datetime('now'))
             ON CONFLICT(ticker) DO UPDATE SET
                growth = excluded.growth,
                desired_return = excluded.desired_return,
                updated_at = excluded.updated_at",
            params![ticker, assumptions.growth, assumptions.desired_return],
        )?;
        Ok(())
    }

    fn get_config(&self, key: &str) -> Result<Option<String>, AppError> {
        let conn = self.conn.lock().expect("store lock");
        conn.query_row(
            "SELECT value FROM config WHERE key = ?1",
            params![key],
            |row| row.get(0),
        )
        .optional()
        .map_err(AppError::from)
    }

    fn set_config(&self, key: &str, value: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().expect("store lock");
        conn.execute(
            "INSERT INTO config (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )?;
        Ok(())
    }

    fn get_cached<T: DeserializeOwned>(
        &self,
        sql: &str,
        ticker: &str,
    ) -> Result<Option<Cached<T>>, AppError> {
        let conn = self.conn.lock().expect("store lock");
        let row = conn
            .query_row(sql, params![ticker], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .optional()?;
        match row {
            None => Ok(None),
            Some((payload, provider, fetched_at)) => Ok(Some(Cached {
                value: serde_json::from_str(&payload)?,
                provider,
                fetched_at,
            })),
        }
    }

    fn put_row<T: Serialize>(
        &self,
        sql: &str,
        ticker: &str,
        value: &T,
        provider: &str,
    ) -> Result<(), AppError> {
        let payload = serde_json::to_string(value)?;
        let conn = self.conn.lock().expect("store lock");
        conn.execute(sql, params![ticker, payload, provider])?;
        Ok(())
    }
}

fn home_dir() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Quote;

    #[test]
    fn watchlist_and_quote_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(&dir.path().join("t.db")).expect("open");
        store.add_watch("ACME").expect("add");
        store.add_watch("ACME").expect("idempotent");
        assert_eq!(store.watchlist().expect("list"), vec!["ACME".to_string()]);
        let quote = Quote {
            ticker: "ACME".into(),
            name: "Acme".into(),
            sector: "Industrials".into(),
            price: 10.0,
            currency: "USD".into(),
            market_cap: None,
            shares_outstanding: None,
        };
        store.put_quote("ACME", &quote, "fixture").expect("put");
        let cached = store.quote("ACME").expect("get").expect("present");
        assert_eq!(cached.value.price, 10.0);
        assert_eq!(cached.provider, "fixture");
        assert!(store.remove_watch("ACME").expect("remove"));
        assert!(store.watchlist().expect("empty").is_empty());
    }

    #[test]
    fn fmp_key_is_stored_and_cleared() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(&dir.path().join("t.db")).expect("open");
        assert_eq!(store.fmp_key().expect("none"), None);
        store.set_fmp_key(Some("secret")).expect("set");
        assert_eq!(store.fmp_key().expect("key").as_deref(), Some("secret"));
        store.set_fmp_key(None).expect("clear");
        assert_eq!(store.fmp_key().expect("cleared"), None);
    }

    #[test]
    fn note_roundtrip_and_clear() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(&dir.path().join("t.db")).expect("open");
        assert_eq!(store.note("ACME").expect("empty"), "");
        store.put_note("ACME", "Buy on weakness").expect("put");
        assert_eq!(store.note("ACME").expect("get"), "Buy on weakness");
        store.put_note("ACME", "").expect("clear");
        assert_eq!(store.note("ACME").expect("cleared"), "");
    }
}
