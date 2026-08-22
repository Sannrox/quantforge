# QuantForge

Local-first research workbench for long-term investors.

Open a ticker, read 10+ years of fundamentals and valuation in clean charts,
run a simple DCF, and keep a short watchlist. The host owns the data. The
browser never sees provider keys.

**Status:** early / local-only. Not a hosted product. Sekai Chisei projections
and Aldunis enterprise hosting are reserved and unimplemented.

## Requirements

- Rust 1.85+ (edition 2024)
- Node.js for the Vite UI toolchain only
- No Keycloak, Aldunis, Chisei, or paid data key required

## Quick start

```sh
make start
```

Open [http://127.0.0.1:4177](http://127.0.0.1:4177). The host listens on
`127.0.0.1:4176` and refuses non-loopback binds.

Add **ACME** on the watchlist for the offline fixture (12 years of statements,
no network).

## Data providers

| Provider | Key | Notes |
| --- | --- | --- |
| `fixture` | none | Ships `testdata/acme.json`. Default. |
| `yahoo` | none | Unofficial Yahoo Finance JSON. May fail or rate-limit. |
| `fmp` | host-side API key | [Financial Modeling Prep](https://site.financialmodelingprep.com/). The key is stored in local SQLite and is never returned to the browser. |

Switch providers in Settings. Refresh is a button (or first open). This is not
a live tape.

## Layout

```text
src/        Rust host
web/        Vite / React desk
testdata/   fixture financials
docs/       product spec and reserved contracts
```

SQLite lives at `~/.quantforge/quantforge.db`.

## License

MIT. See [LICENSE](LICENSE).
