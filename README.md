# QuantForge

Local-first research workbench for long-term investors.

Open any name you might hold for years and, on one local page, decide whether
the business is still high quality, cheap enough, and able to survive — then
write the call and keep a short list. The host owns the data. The browser never
sees provider keys.

**Status:** early / local-only. Not a hosted product.

## Requirements

- [Rust](https://rustup.rs/) 1.85+ (`rustup` installs `stable`)
- [Node.js](https://nodejs.org/) 20+ (desk build only)
- No paid data key, Keycloak, or hosted account

## Quick start

```sh
git clone https://github.com/Sannrox/quantforge.git
cd quantforge
make start
```

Open [http://127.0.0.1:4176](http://127.0.0.1:4176). That is the product: the
host serves the desk. It binds loopback only.

1. Click **Add ACME** (offline, 12 years of statements, no network). Judge
   quality, cheapness, and survival, write the call, save a DCF.
2. Add any live ticker. First open fetches Yahoo. Switch to `fmp` in Settings
   when you want a longer statement history, then Refresh.

Yahoo is unofficial and often returns about four years of statements. That is
enough for a first pass; it is not a 10-year history.

```sh
make check
```

Desk HMR (optional): `make dev` then [http://127.0.0.1:4177](http://127.0.0.1:4177).

## Data providers

| Provider | Key | Notes |
| --- | --- | --- |
| `fixture` | none | ACME is compiled in. Default. |
| `yahoo` | none | Unofficial Yahoo Finance JSON. May fail or rate-limit. |
| `fmp` | host-side API key | [Financial Modeling Prep](https://site.financialmodelingprep.com/). The key is stored in local SQLite and is never returned to the browser. |

Refresh is a button (or first open). This is not a live tape.

## Layout

```text
src/        Rust host
web/        Vite / React desk
testdata/   fixture financials
docs/       product spec and reserved contracts
```

SQLite lives at `~/.quantforge/quantforge.db`.

## License

MIT. See [LICENSE](LICENSE). See [CONTRIBUTING](CONTRIBUTING.md) to work on the
repo.
