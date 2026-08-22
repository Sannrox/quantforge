# QuantForge product spec

QuantForge is a local-first research workbench for long-term investors.

## Outcome

A long-term investor can open any name they might hold for years and, on one
local page, decide whether the business is still high quality, cheap enough,
and able to survive — then write the call and keep a short list — without a
tape, a broker, or a hosted product.

## Job

Show, in clean charts, whether a business is still high quality, cheap enough,
and able to survive a storm — then write the call.

## Surfaces (v1)

1. Watchlist — the rail is navigation: ticker, last price, and one yield line
   (FCF yield on EV when present). Home is the short list of calls: ticker,
   price, quality (FCF/sh CAGR), cheapness (FCF yield vs the saved
   desired-return hurdle), survival (interest cover), and the written call.
   Extra fields stay on the company payload; they are not a second dashboard.
2. Company page — identity, price, market cap, net cash and enterprise value;
   a stale-cache banner when Refresh would fetch a different provider;
   a short-history note when statements cover fewer than eight years; a
   judgment strip (quality: FCF/sh CAGR, operating margin vs median, ROIC vs
   this company’s history, share change; cheapness: FCF yield and FCF yield on
   EV vs the hurdle, P/FCF vs history, DCF vs price; survival: interest cover
   vs this company’s history, net cash); the written call and DCF next; a
   History fold for the longer snapshot; 10+ year charts when the provider has
   them (FCF, ROIC, valuation with P/E, P/FCF, and P/OCF median overlays,
   shares) plus more charts behind a fold (price, OCF, EPS vs FCF/sh, margins,
   net cash, interest cover); annual or quarterly period on charts and
   statements; single-series levels as columns; hover readout with YoY on
   single series; statement tables with year-over-year change including
   operating cash flow, EPS, shares, FCF margin, debt, and interest cover. An
   empty watchlist offers ACME. A live ticker added while Settings is still
   fixture fetches Yahoo on first open via chart, search, and fundamentals
   timeseries. A failed add does not leave a blank
   name on the list.
3. DCF — growth and desired return; seed from TTM free cash flow per share,
   falling back to diluted EPS; persist assumptions per ticker; operating
   stream from a 10-year Gordon model; equity fair value is that stream plus
   net cash per share (or minus net debt) when the balance sheet is present;
   implied multiple on the operating stream, not the cash pile; 3×3
   sensitivity of equity fair value with the saved cell marked.
4. Snapshot — folded under History: 10y and 5y CAGRs including FCF per share
   and 5y−10y fade, median and p25–p75 P/E, P/FCF, and P/OCF vs today, years
   of FCF/sh, EPS, or OCF/sh growth to reach those medians, multiple vs 10y
   high, FCF and OCF yield vs their historical medians, earnings yield vs its
   historical median, FCF yield and FCF power vs the saved desired-return
   hurdle, OCF power vs the same hurdle, FCF yield on enterprise value vs its
   historical median, ROIC vs history including 3y, FCF margin interquartile
   range, share change and annualized share CAGR, FCF conversion vs history
   including 3y, margins vs history, reinvestment vs sales, FCF and revenue
   consistency including sequential FCF and OCF up-years. Yahoo quarters are
   labeled Q1–Q4 from period-end month so year-over-year compares the same
   quarter.
5. Call — a short written call per ticker, stored on the host, and shown on
   the short list.
6. Settings — active provider (`fixture`, `yahoo`, `fmp`); optional FMP key
   stored on the host. Fixture is the ACME demo. Other names use Yahoo until
   you choose Yahoo or FMP for every refresh.

Refresh is explicit or first-open. This is not a live tape.

## Authority

| Concern | Owner |
| --- | --- |
| Quotes, statements, prices, watchlist, DCF assumptions | QuantForge SQLite |
| Policy, provenance, audit, later AI routing | Sekai Chisei, when configured (reserved) |
| Tenant identity and hosted assertions | Aldunis, later (reserved) |

## Non-goals

Portfolio, dividends, earnings calendar, transcripts, AI summaries, real-time
quotes, broker sync, a screener, and implementing the reserved Chisei or
Aldunis clients.
