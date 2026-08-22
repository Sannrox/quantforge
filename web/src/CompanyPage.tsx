import { FormEvent, KeyboardEvent, useEffect, useRef, useState } from "react";
import { api, type Company, type Point, type SeriesSet, type StatementRow } from "./api";
import { Chart } from "./Chart";
import { compact, fmt, money, pct, times } from "./format";

export function CompanyPage({
  ticker,
  onWatchlist,
  onRemove,
  onAdd,
}: {
  ticker: string;
  onWatchlist: () => Promise<void>;
  onRemove: (ticker: string) => Promise<void>;
  onAdd: (ticker: string) => Promise<void>;
}) {
  const [company, setCompany] = useState<Company | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [period, setPeriod] = useState<"annual" | "quarterly">("annual");
  const headingRef = useRef<HTMLHeadingElement>(null);

  useEffect(() => {
    setPeriod("annual");
  }, [ticker]);

  useEffect(() => {
    let cancelled = false;
    setCompany(null);
    setError(null);
    api
      .company(ticker)
      .then((row) => {
        if (!cancelled) {
          setCompany(row);
        }
      })
      .catch((err: Error) => {
        if (!cancelled) {
          setError(err.message);
        }
      })
      .finally(() => onWatchlist().catch(() => undefined));
    return () => {
      cancelled = true;
    };
  }, [onWatchlist, ticker]);

  const focusedTicker = useRef<string | null>(null);
  useEffect(() => {
    if ((company || error) && focusedTicker.current !== ticker) {
      headingRef.current?.focus();
      focusedTicker.current = ticker;
    }
  }, [company, error, ticker]);

  async function onCompanySaved(row: Company) {
    setCompany(row);
    await onWatchlist();
  }

  async function refresh() {
    setBusy(true);
    setError(null);
    try {
      setCompany(await api.refresh(ticker));
      await onWatchlist();
    } catch (err) {
      setError(err instanceof Error ? err.message : "Refresh failed");
    } finally {
      setBusy(false);
    }
  }

  if (error && !company) {
    return (
      <section>
        <p className="kicker">{ticker}</p>
        <h1 ref={headingRef} tabIndex={-1}>
          Unavailable
        </h1>
        <p className="error" role="alert">
          {error}
        </p>
        <p className="note">
          Fixture only ships ACME. For a live ticker, switch the provider to yahoo in{" "}
          <a href="#/settings">Settings</a>, then add the name again.
        </p>
        <div className="actions">
          <button className="btn" type="button" onClick={() => void refresh()}>
            Retry
          </button>
          {ticker !== "ACME" ? (
            <button className="btn" type="button" onClick={() => void onAdd("ACME")}>
              Open ACME
            </button>
          ) : null}
          <button className="btn btn-quiet" type="button" onClick={() => void onRemove(ticker)}>
            Remove
          </button>
        </div>
      </section>
    );
  }
  if (!company) {
    return (
      <p className="note" aria-busy="true">
        Loading {ticker}…
      </p>
    );
  }

  const rows = period === "annual" ? company.annual : company.quarterly;
  const set = period === "annual" ? company.series : company.quarterly_series ?? company.series;
  const hasQuarterly = company.quarterly.length > 0;
  return (
    <article>
      <header className="desk-head">
        <div>
          <p className="kicker">
            {company.sector || "Company"} · {company.provider} · {company.fetched_at}
          </p>
          <h1 ref={headingRef} tabIndex={-1}>
            {company.name} <span className="watch-ticker">{company.ticker}</span>
          </h1>
        </div>
        <div>
          <div className="price">{money(company.price, company.currency)}</div>
          {company.market_cap != null ? <div className="note">Cap {compact(company.market_cap)}</div> : null}
          {company.multiples.net_cash != null || company.multiples.enterprise_value != null ? (
            <div className="note">
              {company.multiples.net_cash != null
                ? `Net cash ${compact(company.multiples.net_cash)}`
                : null}
              {company.multiples.net_cash != null && company.multiples.enterprise_value != null
                ? " · "
                : null}
              {company.multiples.enterprise_value != null
                ? `EV ${compact(company.multiples.enterprise_value)}`
                : null}
            </div>
          ) : null}
          <div className="actions">
            <button className="btn" type="button" disabled={busy} onClick={() => void refresh()}>
              Refresh
            </button>
            <button className="btn btn-quiet" type="button" onClick={() => void onRemove(company.ticker)}>
              Remove
            </button>
          </div>
        </div>
      </header>
      {error ? (
        <p className="error" role="alert">
          {error}
        </p>
      ) : null}
      {company.provider !== company.active_provider ? (
        <p className="banner" role="status">
          This page is cached from {company.provider}. Active provider is {company.active_provider}. Refresh to
          fetch from {company.active_provider}.
        </p>
      ) : null}
      {company.snapshot.years > 0 && company.snapshot.years < 8 ? (
        <p className="note">
          History uses {company.snapshot.years} years of statements. Yahoo often returns about four; FMP usually
          has more.
        </p>
      ) : null}
      <Judgment company={company} />
      <div className="split memo">
        <NotesPanel company={company} onSaved={onCompanySaved} />
        <DcfPanel company={company} onSaved={onCompanySaved} />
      </div>
      <History company={company} />
      <div className="tabs" role="tablist" aria-label="Fiscal period" onKeyDown={onTabKey}>
        <button
          type="button"
          className="btn btn-quiet"
          role="tab"
          id="tab-annual"
          aria-selected={period === "annual"}
          aria-controls="charts-panel"
          tabIndex={period === "annual" ? 0 : -1}
          onClick={() => setPeriod("annual")}
        >
          Annual
        </button>
        <button
          type="button"
          className="btn btn-quiet"
          role="tab"
          id="tab-quarterly"
          aria-selected={period === "quarterly"}
          aria-controls="charts-panel"
          tabIndex={period === "quarterly" ? 0 : -1}
          disabled={!hasQuarterly}
          onClick={() => setPeriod("quarterly")}
        >
          Quarterly
        </button>
      </div>
      <div id="charts-panel" role="tabpanel" aria-labelledby={period === "annual" ? "tab-annual" : "tab-quarterly"}>
        <ChartGrid
          set={set}
          peMedian={company.snapshot.pe_median}
          pFcfMedian={company.snapshot.p_fcf_median}
          pOcfMedian={company.snapshot.p_ocf_median}
          prices={company.price_series ?? []}
        />
      </div>
      <section className="panel">
        <h2 id="statements-heading">{period === "annual" ? "Annual statements" : "Quarterly statements"}</h2>
        <StatementTable rows={rows} period={period} />
      </section>
    </article>
  );
}

function Judgment({ company }: { company: Company }) {
  const snap = company.snapshot;
  return (
    <div className="judgment">
      <section>
        <p className="kicker">Quality</p>
        <dl className="multiples judgment-grid">
          <div className="stat">
            <dt>FCF / sh CAGR</dt>
            <dd>
              {pct(snap.fcf_ps_cagr)} · {pct(snap.fcf_ps_cagr_5y)}
            </dd>
          </div>
          <div className="stat">
            <dt>Op. margin vs median</dt>
            <dd>
              {pct(snap.operating_margin)} · {pct(snap.operating_margin_vs_median)}
            </dd>
          </div>
          <div className="stat">
            <dt>ROIC vs {snap.years}y median {pct(snap.roic_median)}</dt>
            <dd>
              {pct(snap.roic)} · {pct(snap.roic_vs_median)}
            </dd>
          </div>
          <div className="stat">
            <dt>Shares / y</dt>
            <dd>
              {pct(snap.share_change)} · {pct(snap.share_cagr)}
            </dd>
          </div>
        </dl>
      </section>
      <section>
        <p className="kicker">Cheapness</p>
        <dl className="multiples judgment-grid">
          <div className="stat">
            <dt>FCF yield vs {pct(company.assumptions.desired_return)} hurdle</dt>
            <dd>
              {pct(company.multiples.fcf_yield)} · {pct(company.fcf_yield_vs_hurdle)}
            </dd>
          </div>
          <div className="stat">
            <dt>P/FCF vs {snap.years}y median {fmt(snap.p_fcf_median)}</dt>
            <dd>
              {fmt(company.multiples.p_fcf)} · {pct(snap.p_fcf_vs_median)}
            </dd>
          </div>
          <div className="stat">
            <dt>FCF yield on EV vs {pct(company.assumptions.desired_return)} hurdle</dt>
            <dd>
              {pct(company.multiples.fcf_yield_ev)} · {pct(company.fcf_yield_ev_vs_hurdle)}
            </dd>
          </div>
          <div className="stat">
            <dt>DCF vs price</dt>
            <dd>
              {company.dcf
                ? `${money(company.dcf.fair_value, company.currency)} · ${pct(company.dcf.upside)}`
                : "—"}
            </dd>
          </div>
        </dl>
      </section>
      <section>
        <p className="kicker">Survival</p>
        <dl className="multiples judgment-grid">
          <div className="stat">
            <dt>Interest cover vs {snap.years}y median {times(snap.interest_coverage_median)}</dt>
            <dd>
              {times(snap.interest_coverage)} · {pct(snap.interest_coverage_vs_median)}
            </dd>
          </div>
          <div className="stat">
            <dt>Net cash</dt>
            <dd>{compact(company.multiples.net_cash)}</dd>
          </div>
        </dl>
      </section>
    </div>
  );
}

function History({ company }: { company: Company }) {
  const snap = company.snapshot;
  const rows: [string, string][] = [
    [
      `Rev ${snap.years}y / 5y CAGR`,
      `${pct(snap.revenue_cagr)} · ${pct(snap.revenue_cagr_5y)}${fadeBit(snap.revenue_cagr_fade)}`,
    ],
    [
      `FCF ${snap.years}y / 5y CAGR`,
      `${pct(snap.fcf_cagr)} · ${pct(snap.fcf_cagr_5y)}${fadeBit(snap.fcf_cagr_fade)}`,
    ],
    [
      `FCF/sh ${snap.years}y / 5y CAGR`,
      `${pct(snap.fcf_ps_cagr)} · ${pct(snap.fcf_ps_cagr_5y)}${fadeBit(snap.fcf_ps_cagr_fade)}`,
    ],
    [
      `EPS ${snap.years}y / 5y CAGR`,
      `${pct(snap.eps_cagr)} · ${pct(snap.eps_cagr_5y)}${fadeBit(snap.eps_cagr_fade)}`,
    ],
    ["P/E", fmt(company.multiples.pe)],
    ["P/FCF", fmt(company.multiples.p_fcf)],
    ["P/OCF", fmt(company.multiples.p_ocf)],
    ["Net cash", compact(company.multiples.net_cash)],
    ["Enterprise value", compact(company.multiples.enterprise_value)],
    ["EV / FCF", fmt(company.multiples.ev_fcf)],
    [
      `FCF yield on EV vs ${snap.years}y median ${pct(snap.fcf_yield_ev_median)}`,
      `${pct(company.multiples.fcf_yield_ev)} · ${pct(snap.fcf_yield_ev_vs_median)}`,
    ],
    [
      `ROIC vs ${snap.years}y median ${pct(snap.roic_median)}`,
      `${pct(snap.roic)} · ${pct(snap.roic_vs_median)} · 3y ${pct(snap.roic_3y_vs_median)}`,
    ],
    [
      `Interest cover vs ${snap.years}y median ${times(snap.interest_coverage_median)}`,
      `${times(snap.interest_coverage)} · ${pct(snap.interest_coverage_vs_median)} · 3y ${times(snap.interest_coverage_3y)}`,
    ],
    [
      `Earnings yield vs ${snap.years}y median ${pct(snap.earnings_yield_median)}`,
      `${pct(company.multiples.earnings_yield)} · ${pct(snap.earnings_yield_vs_median)}`,
    ],
    ["OCF yield", pct(company.multiples.ocf_yield)],
    [
      "FCF power vs hurdle",
      `${pct(snap.fcf_power)} · ${pct(company.fcf_power_vs_hurdle)}`,
    ],
    [
      "OCF power vs hurdle",
      `${pct(snap.ocf_power)} · ${pct(company.ocf_power_vs_hurdle)}`,
    ],
    [
      `P/E vs ${snap.years}y median ${fmt(snap.pe_median)} · ${fmt(snap.pe_p25)}–${fmt(snap.pe_p75)}`,
      `${pct(snap.pe_vs_median)} · ${pctile(snap.pe_percentile)} · vs high ${pct(snap.pe_vs_high)}${yearsBit(snap.years_to_median_pe)}`,
    ],
    [
      `P/FCF vs ${snap.years}y median ${fmt(snap.p_fcf_median)} · ${fmt(snap.p_fcf_p25)}–${fmt(snap.p_fcf_p75)}`,
      `${pct(snap.p_fcf_vs_median)} · ${pctile(snap.p_fcf_percentile)} · vs high ${pct(snap.p_fcf_vs_high)}${yearsBit(snap.years_to_median_p_fcf)}`,
    ],
    [
      `P/OCF vs ${snap.years}y median ${fmt(snap.p_ocf_median)} · ${fmt(snap.p_ocf_p25)}–${fmt(snap.p_ocf_p75)}`,
      `${pct(snap.p_ocf_vs_median)} · ${pctile(snap.p_ocf_percentile)} · vs high ${pct(snap.p_ocf_vs_high)}${yearsBit(snap.years_to_median_p_ocf)}`,
    ],
    [
      `FCF yield vs ${snap.years}y median ${pct(snap.fcf_yield_median)}`,
      `${pct(company.multiples.fcf_yield)} · ${pct(snap.fcf_yield_vs_median)}`,
    ],
    [
      `OCF yield vs ${snap.years}y median ${pct(snap.ocf_yield_median)}`,
      `${pct(company.multiples.ocf_yield)} · ${pct(snap.ocf_yield_vs_median)}`,
    ],
    [
      `FCF / NI vs ${snap.years}y median ${pct(snap.fcf_conversion_median)}`,
      `${pct(snap.fcf_conversion)} · ${pct(snap.fcf_conversion_vs_median)} · 3y ${pct(snap.fcf_conversion_3y_vs_median)}`,
    ],
    [
      `Gross margin vs ${snap.years}y median ${pct(snap.gross_margin_median)}`,
      `${pct(snap.gross_margin)} · ${pct(snap.gross_margin_vs_median)}`,
    ],
    [
      `Net margin vs ${snap.years}y median ${pct(snap.net_margin_median)}`,
      `${pct(snap.net_margin)} · ${pct(snap.net_margin_vs_median)}`,
    ],
    [
      `Op. margin vs ${snap.years}y median ${pct(snap.operating_margin_median)}`,
      `${pct(snap.operating_margin)} · ${pct(snap.operating_margin_vs_median)} · 3y ${pct(snap.operating_margin_3y_vs_median)}`,
    ],
    [
      `FCF margin vs ${snap.years}y median ${pct(snap.fcf_margin_median)}`,
      `${pct(snap.fcf_margin)} · ${pct(snap.fcf_margin_vs_median)} · 3y ${pct(snap.fcf_margin_3y_vs_median)}${snap.fcf_margin_iqr != null ? ` · IQR ${pct(snap.fcf_margin_iqr)}` : ""}`,
    ],
    [
      `Reinvest / sales vs ${snap.years}y median ${pct(snap.reinvestment_median)}`,
      `${pct(snap.reinvestment)} · ${pct(snap.reinvestment_vs_median)}`,
    ],
    [
      "FCF + · FCF up · OCF up · Rev up",
      `${count(snap.fcf_positive_years, snap.fcf_years)} · ${count(snap.fcf_up_years, snap.fcf_pairs)} · ${count(snap.ocf_up_years, snap.ocf_pairs)} · ${count(snap.revenue_up_years, snap.revenue_pairs)}`,
    ],
  ];
  return (
    <details className="fold">
      <summary>History</summary>
      <dl className="history-list">
        {rows.map(([label, value]) => (
          <div key={label}>
            <dt>{label}</dt>
            <dd>{value}</dd>
          </div>
        ))}
      </dl>
    </details>
  );
}

function ChartGrid({
  set,
  peMedian,
  pFcfMedian,
  pOcfMedian,
  prices,
}: {
  set: SeriesSet;
  peMedian: number | null;
  pFcfMedian: number | null;
  pOcfMedian: number | null;
  prices: Point[];
}) {
  return (
    <>
      <div className="chart-grid">
        <Chart title="Free cash flow" mode="bars" series={[{ label: "FCF", points: set.fcf }]} />
        <Chart title="ROIC" kind="percent" series={[{ label: "ROIC", points: set.roic }]} />
        <Chart
          title="Valuation"
          kind="ratio"
          series={[
            { label: "P/E", points: set.pe },
            { label: "P/FCF", points: set.p_fcf, dashed: true },
            { label: "P/OCF", points: set.p_ocf, dashed: true },
            ...(peMedian != null ? [{ label: "P/E median", points: flat(set.pe, peMedian), dashed: true }] : []),
            ...(pFcfMedian != null
              ? [{ label: "P/FCF median", points: flat(set.p_fcf, pFcfMedian), dashed: true }]
              : []),
            ...(pOcfMedian != null
              ? [{ label: "P/OCF median", points: flat(set.p_ocf, pOcfMedian), dashed: true }]
              : []),
          ]}
        />
        <Chart title="Shares" mode="bars" series={[{ label: "Shares", points: set.shares }]} />
      </div>
      <MoreCharts set={set} prices={prices} />
    </>
  );
}

function MoreCharts({ set, prices }: { set: SeriesSet; prices: Point[] }) {
  const [open, setOpen] = useState(false);
  return (
    <details className="fold" onToggle={(event) => setOpen(event.currentTarget.open)}>
      <summary>More charts</summary>
      {open ? (
        <div className="chart-grid">
          {prices.length > 0 ? <Chart title="Price" series={[{ label: "Price", points: prices }]} /> : null}
          <Chart title="Revenue" mode="bars" series={[{ label: "Revenue", points: set.revenue }]} />
          <Chart
            title="Margins"
            kind="percent"
            series={[
              { label: "Gross", points: set.gross_margin },
              { label: "Operating", points: set.operating_margin, dashed: true },
              { label: "Net", points: set.net_margin, dashed: true },
              { label: "FCF", points: set.fcf_margin, dashed: true },
            ]}
          />
          <Chart title="Net cash" mode="bars" series={[{ label: "Net cash", points: set.net_cash }]} />
          <Chart
            title="Interest cover"
            kind="ratio"
            series={[{ label: "EBIT / interest", points: set.interest_coverage }]}
          />
          <Chart title="Operating cash flow" mode="bars" series={[{ label: "OCF", points: set.ocf }]} />
          <Chart title="FCF / share" mode="bars" series={[{ label: "FCF / sh", points: set.fcf_ps }]} />
          <Chart
            title="Cash conversion"
            kind="percent"
            series={[
              { label: "FCF / NI", points: set.fcf_conversion },
              { label: "Reinvest / sales", points: set.reinvestment, dashed: true },
            ]}
          />
          <Chart
            title="EPS · FCF / sh"
            series={[
              { label: "EPS", points: set.eps },
              { label: "FCF / sh", points: set.fcf_ps, dashed: true },
            ]}
          />
          <Chart title="EBITDA" mode="bars" series={[{ label: "EBITDA", points: set.ebitda }]} />
        </div>
      ) : null}
    </details>
  );
}

function onTabKey(event: KeyboardEvent<HTMLDivElement>) {
  if (event.key !== "ArrowRight" && event.key !== "ArrowLeft") {
    return;
  }
  event.preventDefault();
  const tabs = [...event.currentTarget.querySelectorAll<HTMLButtonElement>('[role="tab"]:not(:disabled)')];
  const index = tabs.indexOf(document.activeElement as HTMLButtonElement);
  const next = tabs[(index + (event.key === "ArrowRight" ? 1 : tabs.length - 1)) % tabs.length];
  next?.focus();
  next?.click();
}

function StatementTable({ rows, period }: { rows: StatementRow[]; period: string }) {
  return (
    <div className="table-wrap">
      <table>
        <caption className="sr-only">{period} statements with year-over-year change</caption>
        <thead>
          <tr>
            <th scope="col">Period</th>
            <th scope="col">Revenue</th>
            <th scope="col">NI</th>
            <th scope="col">OCF</th>
            <th scope="col">FCF</th>
            <th scope="col">EPS</th>
            <th scope="col">Shares</th>
            <th scope="col">Op. margin</th>
            <th scope="col">FCF margin</th>
            <th scope="col">Debt</th>
            <th scope="col">Cover</th>
          </tr>
        </thead>
        <tbody>
          {rows.map((row) => (
            <tr key={`${row.period_end}-${row.fiscal_period}`}>
              <th scope="row">{statementLabel(row)}</th>
              <td>{amount(row.revenue, row.revenue_yoy, compact)}</td>
              <td>{amount(row.net_income, row.net_income_yoy, compact)}</td>
              <td>{amount(row.operating_cash_flow, row.operating_cash_flow_yoy, compact)}</td>
              <td>{amount(row.free_cash_flow, row.free_cash_flow_yoy, compact)}</td>
              <td>{amount(row.eps, row.eps_yoy, fmt)}</td>
              <td>{amount(row.shares_outstanding, row.shares_yoy, compact)}</td>
              <td>
                {pct(row.operating_margin)}
                {row.operating_margin_yoy != null ? (
                  <span className="yoy"> {pp(row.operating_margin_yoy)}</span>
                ) : null}
              </td>
              <td>
                {pct(row.fcf_margin)}
                {row.fcf_margin_yoy != null ? (
                  <span className="yoy"> {pp(row.fcf_margin_yoy)}</span>
                ) : null}
              </td>
              <td>{compact(row.debt)}</td>
              <td>{times(row.interest_coverage)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function DcfPanel({ company, onSaved }: { company: Company; onSaved: (company: Company) => void }) {
  const [growth, setGrowth] = useState(percentString(company.assumptions.growth));
  const [ret, setRet] = useState(percentString(company.assumptions.desired_return));
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    setGrowth(percentString(company.assumptions.growth));
    setRet(percentString(company.assumptions.desired_return));
  }, [company.assumptions.desired_return, company.assumptions.growth, company.ticker]);

  async function save(event: FormEvent) {
    event.preventDefault();
    const growthRate = Number(growth) / 100;
    const desired = Number(ret) / 100;
    if (!Number.isFinite(growthRate) || growthRate < 0 || growthRate > 1) {
      setError("Growth must be between 0 and 100.");
      return;
    }
    if (!Number.isFinite(desired) || desired < 0.01 || desired > 1) {
      setError("Desired return must be between 1 and 100.");
      return;
    }
    if (desired <= growthRate) {
      setError("Desired return must be greater than growth.");
      return;
    }
    setError(null);
    try {
      onSaved(await api.saveDcf(company.ticker, growthRate, desired));
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not save DCF");
    }
  }

  const dcf = company.dcf;
  return (
    <section className="panel">
      <h2>DCF</h2>
      <form className="dcf-form" onSubmit={(event) => void save(event)}>
        <label className="field" htmlFor="dcf-growth">
          Growth %
          <input
            id="dcf-growth"
            value={growth}
            onChange={(event) => {
              setGrowth(event.target.value);
              setError(null);
            }}
            inputMode="decimal"
          />
        </label>
        <label className="field" htmlFor="dcf-return">
          Desired return %
          <input
            id="dcf-return"
            value={ret}
            onChange={(event) => {
              setRet(event.target.value);
              setError(null);
            }}
            inputMode="decimal"
          />
        </label>
        <button className="btn" type="submit">
          Save assumptions
        </button>
      </form>
      {error ? (
        <p className="error" role="alert">
          {error}
        </p>
      ) : null}
      {dcf ? (
        <div className="dcf-out">
          <span className="note">
            Fair value from TTM {dcf.seed_kind.toUpperCase()} / share ({money(dcf.seed_per_share, company.currency)}),{" "}
            {dcf.years} years
          </span>
          <strong>{money(dcf.fair_value, company.currency)}</strong>
          <span className="note">{bridgeLine(dcf, company.currency)}</span>
          <span className="note">
            vs {money(dcf.price, company.currency)}
            {dcf.upside != null ? ` · ${pct(dcf.upside)} vs price` : ""}
            {impliedMultiple(dcf) != null
              ? ` · implies ${fmt(impliedMultiple(dcf))}x ${dcf.seed_kind.toUpperCase()}`
              : ""}
          </span>
        </div>
      ) : (
        <p className="note">Not enough FCF or EPS to seed a DCF.</p>
      )}
      {company.sensitivity ? (
        <SensitivityTable
          grid={company.sensitivity}
          currency={company.currency}
          savedGrowth={company.assumptions.growth}
          desiredReturn={company.assumptions.desired_return}
        />
      ) : null}
    </section>
  );
}

function impliedMultiple(dcf: NonNullable<Company["dcf"]>): number | null {
  if (!(dcf.seed_per_share > 0)) {
    return null;
  }
  return dcf.stream_value / dcf.seed_per_share;
}

function bridgeLine(dcf: NonNullable<Company["dcf"]>, currency: string): string {
  const operating = `Operating ${money(dcf.stream_value, currency)}`;
  if (dcf.net_cash_per_share == null) {
    return `${operating}. Net cash unavailable.`;
  }
  if (dcf.net_cash_per_share >= 0) {
    return `${operating} + net cash ${money(dcf.net_cash_per_share, currency)} = ${money(dcf.fair_value, currency)}`;
  }
  return `${operating} − net debt ${money(-dcf.net_cash_per_share, currency)} = ${money(dcf.fair_value, currency)}`;
}

function SensitivityTable({
  grid,
  currency,
  savedGrowth,
  desiredReturn,
}: {
  grid: NonNullable<Company["sensitivity"]>;
  currency: string;
  savedGrowth: number;
  desiredReturn: number;
}) {
  return (
    <div className="table-wrap sensitivity">
      <table>
        <caption className="sr-only">DCF fair value by growth and desired return</caption>
        <thead>
          <tr>
            <th scope="col">Return \ Growth</th>
            {grid.growths.map((columnGrowth) => (
              <th key={columnGrowth} scope="col">
                {pct(columnGrowth)}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {grid.returns.map((desired) => (
            <tr key={desired}>
              <th scope="row">{pct(desired)}</th>
              {grid.growths.map((columnGrowth) => {
                const cell = grid.cells.find(
                  (row) =>
                    Math.abs(row.growth - columnGrowth) < 1e-9 &&
                    Math.abs(row.desired_return - desired) < 1e-9,
                );
                const isNow =
                  Math.abs(columnGrowth - savedGrowth) < 1e-9 &&
                  Math.abs(desired - desiredReturn) < 1e-9;
                return (
                  <td key={`${desired}-${columnGrowth}`} className={isNow ? "is-now" : undefined}>
                    {cell?.fair_value != null ? money(cell.fair_value, currency) : "—"}
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

function NotesPanel({ company, onSaved }: { company: Company; onSaved: (company: Company) => void }) {
  const [body, setBody] = useState(company.note);
  const [error, setError] = useState<string | null>(null);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    setBody(company.note);
  }, [company.note, company.ticker]);

  useEffect(() => {
    setSaved(false);
  }, [company.ticker]);

  async function save(event: FormEvent) {
    event.preventDefault();
    if (body.length > 4000) {
      setError("Note must be at most 4000 characters.");
      return;
    }
    setError(null);
    setSaved(false);
    try {
      onSaved(await api.saveNote(company.ticker, body));
      setSaved(true);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Could not save note");
    }
  }

  return (
    <section className="panel">
      <h2>Note</h2>
      <form className="dcf-form" onSubmit={(event) => void save(event)}>
        <label className="field" htmlFor="research-note">
          Thesis
          <textarea
            id="research-note"
            value={body}
            maxLength={4000}
            rows={6}
            onChange={(event) => {
              setBody(event.target.value);
              setSaved(false);
            }}
          />
        </label>
        <button className="btn" type="submit">
          Save note
        </button>
      </form>
      {saved ? (
        <p className="note" role="status">
          Saved.
        </p>
      ) : null}
      {error ? (
        <p className="error" role="alert">
          {error}
        </p>
      ) : null}
    </section>
  );
}

function percentString(value: number): string {
  return String(Math.round(value * 1000) / 10);
}

function fadeBit(value: number | null): string {
  return value == null ? "" : ` · 5y−long ${pct(value)}`;
}

function statementLabel(row: StatementRow): string {
  const year4 = row.period_end.slice(0, 4);
  const year2 = row.period_end.slice(2, 4);
  if (row.fiscal_period === "FY" || row.fiscal_period === "TTM") {
    return year4;
  }
  if (row.fiscal_period.startsWith("Q")) {
    return `${row.fiscal_period}'${year2}`;
  }
  return row.period_end.slice(0, 7);
}

function pp(value: number): string {
  const points = Math.round(value * 1000) / 10;
  return `${points}pp`;
}

function amount(
  value: number | null,
  yoy: number | null,
  format: (value: number | null) => string,
) {
  return (
    <>
      {format(value)}
      {yoy != null ? <span className="yoy"> {pct(yoy)}</span> : null}
    </>
  );
}

function yearsBit(value: number | null): string {
  if (value == null) {
    return "";
  }
  if (value <= 0) {
    return " · at/below median";
  }
  return ` · ~${value.toFixed(1)}y to median`;
}

function pctile(value: number | null): string {
  return value == null ? "—" : `${pct(value)}ile`;
}

function count(numer: number, denom: number): string {
  return denom > 0 ? `${numer}/${denom}` : "—";
}

function flat(points: Point[], value: number): Point[] {
  return points.map((point) => ({ ...point, value }));
}
