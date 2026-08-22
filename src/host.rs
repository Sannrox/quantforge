use std::net::{IpAddr, SocketAddr};
use std::path::PathBuf;
use std::sync::Arc;

use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::{delete, get, post, put};
use axum::{Json, Router};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::domain::DcfAssumptions;
use crate::error::AppError;
use crate::service::{AppState, CompanyView, SettingsUpdate, SettingsView, WatchItem};

pub struct ServeOptions {
    pub bind: IpAddr,
    pub port: u16,
    pub db: PathBuf,
    pub testdata: PathBuf,
    pub web_dir: Option<PathBuf>,
}

pub fn parse_bind(value: &str) -> Result<IpAddr, AppError> {
    let addr: IpAddr = value
        .parse()
        .map_err(|_| AppError::BadRequest(format!("invalid bind address {value}")))?;
    if !addr.is_loopback() {
        return Err(AppError::BadRequest(
            "host binds loopback only (127.0.0.1 or ::1)".into(),
        ));
    }
    Ok(addr)
}

pub fn router(state: Arc<AppState>, web_dir: Option<PathBuf>) -> Router {
    let api = Router::new()
        .route("/healthz", get(healthz))
        .route("/settings", get(get_settings).put(put_settings))
        .route("/watchlist", get(get_watchlist).post(post_watchlist))
        .route("/watchlist/{ticker}", delete(delete_watch))
        .route("/companies/{ticker}", get(get_company))
        .route("/companies/{ticker}/refresh", post(refresh_company))
        .route("/companies/{ticker}/dcf", put(put_dcf))
        .route("/companies/{ticker}/notes", put(put_note));
    let app = Router::new().nest("/api", api).with_state(state);
    if let Some(web_dir) = web_dir.filter(|path| path.exists()) {
        app.fallback_service(ServeDir::new(web_dir))
    } else {
        app.route("/", get(desk_missing))
    }
}

pub async fn serve(options: ServeOptions) -> Result<(), AppError> {
    let store = crate::store::Store::open(&options.db)?;
    let state = Arc::new(AppState::new(store, options.testdata)?);
    let has_desk = options.web_dir.as_ref().is_some_and(|path| path.exists());
    let app = router(state, options.web_dir);
    let addr = SocketAddr::new(options.bind, options.port);
    let listener = TcpListener::bind(addr)
        .await
        .map_err(|error| AppError::Message(format!("bind {addr}: {error}")))?;
    eprintln!("QuantForge listening on http://{addr}");
    if !has_desk {
        eprintln!("Desk not built. Run `make start` from the repo, or open that URL for instructions.");
    }
    axum::serve(listener, app)
        .await
        .map_err(|error| AppError::Message(format!("serve: {error}")))
}

async fn desk_missing() -> Html<&'static str> {
    Html(
        "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>QuantForge</title></head><body>\
         <p>QuantForge host is up. The desk is not in this binary.</p>\
         <p>From the repo: <code>make start</code> then open this same address.</p>\
         <p>That builds the desk and serves it here. ACME is the offline demo. A live ticker fetches Yahoo on first open.</p>\
         </body></html>",
    )
}

async fn healthz() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "ok": true }))
}

async fn get_settings(State(state): State<Arc<AppState>>) -> Result<Json<SettingsView>, AppError> {
    Ok(Json(state.settings()?))
}

async fn put_settings(
    State(state): State<Arc<AppState>>,
    Json(update): Json<SettingsUpdate>,
) -> Result<Json<SettingsView>, AppError> {
    Ok(Json(state.update_settings(update)?))
}

async fn get_watchlist(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<WatchItem>>, AppError> {
    Ok(Json(state.watchlist()?))
}

#[derive(serde::Deserialize)]
struct AddWatch {
    ticker: String,
}

async fn post_watchlist(
    State(state): State<Arc<AppState>>,
    Json(body): Json<AddWatch>,
) -> Result<Json<Vec<WatchItem>>, AppError> {
    Ok(Json(state.add_watch(&body.ticker).await?))
}

async fn delete_watch(
    State(state): State<Arc<AppState>>,
    Path(ticker): Path<String>,
) -> Result<Json<Vec<WatchItem>>, AppError> {
    Ok(Json(state.remove_watch(&ticker)?))
}

async fn get_company(
    State(state): State<Arc<AppState>>,
    Path(ticker): Path<String>,
) -> Result<Json<CompanyView>, AppError> {
    Ok(Json(state.company(&ticker, false).await?))
}

async fn refresh_company(
    State(state): State<Arc<AppState>>,
    Path(ticker): Path<String>,
) -> Result<Json<CompanyView>, AppError> {
    Ok(Json(state.company(&ticker, true).await?))
}

async fn put_dcf(
    State(state): State<Arc<AppState>>,
    Path(ticker): Path<String>,
    Json(assumptions): Json<DcfAssumptions>,
) -> Result<Json<CompanyView>, AppError> {
    Ok(Json(state.save_dcf(&ticker, assumptions)?))
}

#[derive(serde::Deserialize)]
struct NoteBody {
    body: String,
}

async fn put_note(
    State(state): State<Arc<AppState>>,
    Path(ticker): Path<String>,
    Json(note): Json<NoteBody>,
) -> Result<Json<CompanyView>, AppError> {
    Ok(Json(state.save_note(&ticker, &note.body)?))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::Store;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    fn testdata() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")
    }

    async fn test_app() -> Router {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(&dir.path().join("t.db")).expect("store");
        let state = Arc::new(AppState::new(store, testdata()).expect("state"));
        // leak tempdir for test process lifetime
        std::mem::forget(dir);
        router(state, None)
    }

    #[tokio::test]
    async fn root_explains_how_to_build_the_desk() {
        let app = test_app().await;
        let response = app
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        let body = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(body.contains("make start"));
        assert!(body.contains("ACME"));
    }

    #[tokio::test]
    async fn built_desk_is_served_from_the_host() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("index.html"), "<!doctype html><title>desk</title>")
            .expect("index");
        let store = Store::open(&dir.path().join("t.db")).expect("store");
        let state = Arc::new(AppState::new(store, testdata()).expect("state"));
        let app = router(state, Some(dir.path().to_path_buf()));
        let response = app
            .oneshot(Request::get("/").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response.into_body().collect().await.unwrap().to_bytes();
        assert!(String::from_utf8(bytes.to_vec()).unwrap().contains("desk"));
    }

    #[tokio::test]
    async fn fixture_company_and_dcf() {
        let app = test_app().await;
        let add = app
            .clone()
            .oneshot(
                Request::post("/api/watchlist")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"ticker":"acme"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(add.status(), StatusCode::OK);
        let listed = app
            .clone()
            .oneshot(Request::get("/api/watchlist").body(Body::empty()).unwrap())
            .await
            .unwrap();
        assert_eq!(listed.status(), StatusCode::OK);
        let bytes = listed.into_body().collect().await.unwrap().to_bytes();
        let list: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert!(list[0]["pe_vs_median"].as_f64().is_some());
        assert!(list[0]["revenue_cagr"].as_f64().unwrap() > 0.0);
        assert!(list[0]["fcf_yield"].as_f64().unwrap() > 0.0);
        assert!(list[0]["fcf_ps_cagr"].as_f64().unwrap() > 0.0);
        assert!(list[0]["interest_coverage"].as_f64().unwrap() > 1.0);
        assert!(list[0]["net_cash"].as_f64().is_some());
        assert!(list[0]["fcf_yield_vs_median"].as_f64().is_some());
        assert!(list[0]["fcf_yield_vs_hurdle"].as_f64().is_some());
        assert!(list[0]["fcf_power_vs_hurdle"].as_f64().is_some());
        assert!(list[0]["ocf_power_vs_hurdle"].as_f64().is_some());
        assert!(list[0]["years_to_median_p_fcf"].as_f64().is_some());
        assert!(list[0]["years_to_median_pe"].as_f64().is_some());
        assert!(list[0]["fcf_conversion"].as_f64().unwrap() > 0.0);
        assert!(list[0]["pe_percentile"].as_f64().is_some());
        assert!(list[0]["revenue_cagr_5y"].as_f64().unwrap() > 0.0);
        assert!(list[0]["revenue_cagr_fade"].as_f64().is_some());
        let company = app
            .clone()
            .oneshot(
                Request::get("/api/companies/ACME")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(company.status(), StatusCode::OK);
        let bytes = company.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["ticker"], "ACME");
        assert_eq!(body["provider"], "fixture");
        assert_eq!(body["active_provider"], "fixture");
        assert!(body["annual"].as_array().unwrap().len() >= 10);
        assert!(body["series"]["revenue"].as_array().unwrap().len() >= 10);
        assert!(body["dcf"]["fair_value"].as_f64().unwrap() > 0.0);
        let stream = body["dcf"]["stream_value"].as_f64().unwrap();
        let cash_ps = body["dcf"]["net_cash_per_share"].as_f64().unwrap();
        let fair = body["dcf"]["fair_value"].as_f64().unwrap();
        assert!((cash_ps - 15_000_000_000.0 / 1_993_000_000.0).abs() < 1e-6);
        assert!((fair - (stream + cash_ps)).abs() < 1e-6);
        assert!(fair > stream);
        assert!(body["snapshot"]["years"].as_u64().unwrap() >= 10);
        assert!(body["snapshot"]["revenue_cagr"].as_f64().unwrap() > 0.0);
        assert!(body["series"]["shares"].as_array().unwrap().len() >= 10);
        assert!(body["sensitivity"]["cells"].as_array().unwrap().len() >= 4);
        assert!(body["snapshot"]["operating_margin"].as_f64().is_some());
        assert!(body["snapshot"]["revenue_cagr_5y"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["pe_p25"].as_f64().is_some());
        assert!(body["snapshot"]["pe_percentile"].as_f64().is_some());
        assert!(body["snapshot"]["fcf_years"].as_u64().unwrap() >= 10);
        assert!(body["snapshot"]["fcf_ps_cagr"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["revenue_cagr_fade"].as_f64().is_some());
        assert!(body["snapshot"]["pe_high"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["fcf_yield_median"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["share_cagr"].as_f64().is_some());
        assert!(body["snapshot"]["net_margin"].as_f64().is_some());
        assert!(body["fcf_yield_vs_hurdle"].as_f64().is_some());
        assert!(body["fcf_power_vs_hurdle"].as_f64().is_some());
        assert!(body["ocf_power_vs_hurdle"].as_f64().is_some());
        assert!(body["snapshot"]["fcf_power"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["ocf_power"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["fcf_margin_iqr"].as_f64().is_some());
        assert!(body["snapshot"]["years_to_median_p_fcf"].as_f64().is_some());
        assert!(body["snapshot"]["years_to_median_pe"].as_f64().is_some());
        assert!(body["snapshot"]["years_to_median_p_ocf"].as_f64().is_some());
        assert!(body["snapshot"]["ocf_yield_median"].as_f64().unwrap() > 0.0);
        assert!(body["annual"][0]["revenue_yoy"].as_f64().unwrap() > 0.0);
        assert!(body["quarterly"][0]["revenue_yoy"].as_f64().unwrap() > 0.0);
        assert!(body["multiples"]["ocf_yield"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["operating_margin_3y"].as_f64().is_some());
        assert!(body["snapshot"]["fcf_pairs"].as_u64().unwrap() >= 10);
        assert!(body["snapshot"]["reinvestment"].as_f64().is_some());
        assert!(body["series"]["fcf_ps"].as_array().unwrap().len() >= 10);
        assert!(body["multiples"]["fcf_yield"].as_f64().unwrap() > 0.0);
        assert!(body["multiples"]["net_cash"].as_f64().unwrap() > 0.0);
        assert!(body["multiples"]["enterprise_value"].as_f64().unwrap() > 0.0);
        assert!(body["multiples"]["fcf_yield_ev"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["roic"].as_f64().unwrap() > 0.0);
        assert!(body["snapshot"]["interest_coverage"].as_f64().unwrap() > 1.0);
        assert!(body["annual"][0]["interest_coverage"].as_f64().unwrap() > 1.0);
        assert!(body["annual"][0]["debt"].as_f64().unwrap() > 0.0);
        assert!(body["series"]["roic"].as_array().unwrap().len() >= 10);
        assert_eq!(body["note"], "");

        let noted = app
            .clone()
            .oneshot(
                Request::put("/api/companies/ACME/notes")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"body":"Quality compounder"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(noted.status(), StatusCode::OK);
        let bytes = noted.into_body().collect().await.unwrap().to_bytes();
        let noted_body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(noted_body["note"], "Quality compounder");
        let listed = app
            .clone()
            .oneshot(Request::get("/api/watchlist").body(Body::empty()).unwrap())
            .await
            .unwrap();
        let bytes = listed.into_body().collect().await.unwrap().to_bytes();
        let list: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(list[0]["note"], "Quality compounder");

        let saved = app
            .clone()
            .oneshot(
                Request::put("/api/companies/ACME/dcf")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"growth":0.1,"desired_return":0.15}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(saved.status(), StatusCode::OK);
        let bytes = saved.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["assumptions"]["growth"], 0.1);

        let invalid = app
            .oneshot(
                Request::put("/api/companies/ACME/dcf")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"growth":0.2,"desired_return":0.1}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(invalid.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn settings_never_return_fmp_key() {
        let app = test_app().await;
        let saved = app
            .clone()
            .oneshot(
                Request::put("/api/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(
                        r#"{"provider":"fixture","fmp_key":"super-secret"}"#,
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(saved.status(), StatusCode::OK);
        let bytes = saved.into_body().collect().await.unwrap().to_bytes();
        let text = String::from_utf8(bytes.to_vec()).unwrap();
        assert!(!text.contains("super-secret"));
        let body: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(body["has_fmp_key"], true);
        assert!(body.get("fmp_key").is_none());
    }

    #[tokio::test]
    async fn company_reports_stale_cache_when_provider_changes() {
        let app = test_app().await;
        let add = app
            .clone()
            .oneshot(
                Request::post("/api/watchlist")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"ticker":"ACME"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(add.status(), StatusCode::OK);
        let switched = app
            .clone()
            .oneshot(
                Request::put("/api/settings")
                    .header("content-type", "application/json")
                    .body(Body::from(r#"{"provider":"yahoo"}"#))
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(switched.status(), StatusCode::OK);
        let company = app
            .oneshot(
                Request::get("/api/companies/ACME")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(company.status(), StatusCode::OK);
        let bytes = company.into_body().collect().await.unwrap().to_bytes();
        let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(body["provider"], "fixture");
        assert_eq!(body["active_provider"], "yahoo");
    }

    #[test]
    fn rejects_non_loopback() {
        assert!(parse_bind("0.0.0.0").is_err());
        assert!(parse_bind("127.0.0.1").is_ok());
    }
}
