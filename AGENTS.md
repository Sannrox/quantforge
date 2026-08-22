# QuantForge

Local-first research workbench. The host owns quotes, statements, derived
multiples, DCF assumptions, and the watchlist. The browser never receives
provider keys or database paths.

## Layout

- `src/` — Rust host (store, providers, research, valuation, HTTP)
- `web/` — Vite/React desk
- `testdata/` — fixture financials for offline tests
- `docs/` — product spec and reserved plane contracts

## Rules

- Bind loopback only. Do not serve on `0.0.0.0`.
- Persist in QuantForge SQLite. Do not share Aldunis, Chisei, or Tenkai databases.
- Provider keys stay in the host store. Settings responses may say whether a key exists, never the key.
- Chisei and Aldunis hosted mode are reserved contracts, not runtime dependencies.
- Family monochrome tokens only. Charts use weight and dash, not hue.
- Refresh is explicit or first-open. This is not a live tape.
