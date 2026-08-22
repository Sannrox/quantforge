import { FormEvent, useCallback, useEffect, useState } from "react";
import { api, type WatchItem } from "./api";
import { CompanyPage } from "./CompanyPage";
import { SettingsPage } from "./SettingsPage";
import { fmt, money, pct } from "./format";

type Route = { name: "home" } | { name: "company"; ticker: string } | { name: "settings" };

function readRoute(): Route {
  const hash = window.location.hash.replace(/^#/, "");
  if (hash === "/settings") {
    return { name: "settings" };
  }
  const company = hash.match(/^\/c\/([^/]+)$/);
  if (company) {
    try {
      return { name: "company", ticker: decodeURIComponent(company[1]).toUpperCase() };
    } catch {
      return { name: "home" };
    }
  }
  return { name: "home" };
}

export function App() {
  const [route, setRoute] = useState<Route>(readRoute);
  const [watchlist, setWatchlist] = useState<WatchItem[]>([]);
  const [addError, setAddError] = useState<string | null>(null);

  useEffect(() => {
    const onHash = () => setRoute(readRoute());
    window.addEventListener("hashchange", onHash);
    return () => window.removeEventListener("hashchange", onHash);
  }, []);

  useEffect(() => {
    setAddError(null);
  }, [route]);

  const reloadWatch = useCallback(async () => {
    setWatchlist(await api.watchlist());
  }, []);

  useEffect(() => {
    reloadWatch().catch((err: Error) => setAddError(err.message));
  }, [reloadWatch]);

  async function addTicker(ticker: string, form?: HTMLFormElement) {
    const trimmed = ticker.trim();
    if (!trimmed) {
      setAddError("Enter a ticker.");
      return;
    }
    if (trimmed.length > 16 || !/^[A-Za-z0-9.-]+$/.test(trimmed) || /^[.-]+$/.test(trimmed)) {
      setAddError("Ticker must be 1–16 letters, digits, '.', or '-'.");
      return;
    }
    setAddError(null);
    try {
      setWatchlist(await api.addWatch(trimmed));
      form?.reset();
      window.location.hash = `#/c/${trimmed.toUpperCase()}`;
    } catch (err) {
      setAddError(err instanceof Error ? err.message : "Could not add ticker");
    }
  }

  async function onAddSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = event.currentTarget;
    const ticker = String(new FormData(form).get("ticker") ?? "");
    await addTicker(ticker, form);
  }

  async function removeTicker(ticker: string) {
    try {
      setWatchlist(await api.removeWatch(ticker));
    } catch {
      await reloadWatch().catch(() => undefined);
    }
    if (route.name === "company" && route.ticker === ticker) {
      window.location.hash = "#/";
    }
  }

  return (
    <div className="app-shell">
      <a
        className="skip-link"
        href="#desk"
        onClick={(event) => {
          event.preventDefault();
          document.getElementById("desk")?.focus();
        }}
      >
        Skip to research
      </a>
      <aside className="rail">
        <a className="brand" href="#/">
          <span className="brand-name">QuantForge</span>
          <span className="brand-mark">Research</span>
        </a>
        <form className="rail-form" onSubmit={(event) => void onAddSubmit(event)}>
          <input
            name="ticker"
            aria-label="Add ticker"
            placeholder="Add ticker"
            autoComplete="off"
            maxLength={16}
            required
            onChange={() => setAddError(null)}
          />
          <button className="btn" type="submit">
            Add
          </button>
        </form>
        {addError ? (
          <p className="rail-error" role="alert">
            {addError}
          </p>
        ) : null}
        {watchlist.length === 0 ? <p className="rail-empty">No names yet</p> : null}
        <ul className="watch-list">
          {watchlist.map((item) => {
            const active = route.name === "company" && route.ticker === item.ticker;
            return (
              <li key={item.ticker}>
                <div className={`watch-item${active ? " active" : ""}`}>
                  <button
                    type="button"
                    className="watch-row"
                    aria-current={active ? "page" : undefined}
                    onClick={() => {
                      window.location.hash = `#/c/${item.ticker}`;
                    }}
                  >
                    <span className="watch-ticker">{item.ticker}</span>
                    <span className="watch-meta">{item.price != null ? money(item.price, item.currency) : "—"}</span>
                    <span className="watch-name">{item.name ?? "No cache yet"}</span>
                    {item.provider ? <span className="watch-name watch-hint">{item.provider}</span> : null}
                    <span className="watch-meta watch-hint">{watchHint(item)}</span>
                    {item.note ? <span className="watch-name watch-hint">{item.note}</span> : null}
                  </button>
                  <button
                    type="button"
                    className="watch-remove"
                    aria-label={`Remove ${item.ticker}`}
                    onClick={() => void removeTicker(item.ticker)}
                  >
                    Remove
                  </button>
                </div>
              </li>
            );
          })}
        </ul>
        <div className="rail-foot">
          <a href="#/settings" aria-current={route.name === "settings" ? "page" : undefined}>
            Settings
          </a>
          <span className="note">Loopback</span>
        </div>
      </aside>
      <main className="desk" id="desk" tabIndex={-1}>
        {route.name === "settings" ? (
          <SettingsPage />
        ) : route.name === "company" ? (
          <CompanyPage
            ticker={route.ticker}
            onWatchlist={reloadWatch}
            onRemove={removeTicker}
            onAdd={(ticker) => addTicker(ticker)}
          />
        ) : (
          <Home items={watchlist} onAddAcme={() => addTicker("ACME")} />
        )}
      </main>
    </div>
  );
}

function watchHint(item: WatchItem): string {
  if (item.fcf_yield_ev != null) {
    return `FCF yld EV ${pct(item.fcf_yield_ev)}`;
  }
  if (item.fcf_yield_vs_hurdle != null) {
    return `FCF yld ${pct(item.fcf_yield)} · vs hurdle ${pct(item.fcf_yield_vs_hurdle)}`;
  }
  if (item.fcf_yield != null) {
    return `FCF yld ${pct(item.fcf_yield)}`;
  }
  if (item.p_fcf != null) {
    return `P/FCF ${fmt(item.p_fcf)}`;
  }
  return "No history yet";
}

function Home({ items, onAddAcme }: { items: WatchItem[]; onAddAcme: () => Promise<void> }) {
  return (
    <section>
      <p className="kicker">Watchlist</p>
      <h1>Names you are studying</h1>
      <p className="note">
        Open a name, judge quality, cheapness, and survival, write the call. ACME is offline. Any other ticker
        fetches Yahoo on first open.
      </p>
      {items.length === 0 ? (
        <div className="empty-desk">
          <p className="empty">The list is empty.</p>
          <button className="btn" type="button" onClick={() => void onAddAcme()}>
            Add ACME
          </button>
          <p className="note">ACME is the offline fixture: 12 years of statements, no network.</p>
        </div>
      ) : (
        <div className="table-wrap home-table">
          <table>
            <caption className="sr-only">Watchlist quality and cheapness</caption>
            <thead>
              <tr>
                <th scope="col">Ticker</th>
                <th scope="col">Price</th>
                <th scope="col">FCF yield</th>
                <th scope="col">FCF yld EV</th>
                <th scope="col">vs hurdle</th>
                <th scope="col">Note</th>
              </tr>
            </thead>
            <tbody>
              {items.map((item) => (
                <tr key={item.ticker}>
                  <th scope="row">
                    <a href={`#/c/${item.ticker}`}>{item.ticker}</a>
                    {item.name ? <div className="watch-name">{item.name}</div> : null}
                  </th>
                  <td>{item.price != null ? money(item.price, item.currency) : "—"}</td>
                  <td>{pct(item.fcf_yield)}</td>
                  <td>{pct(item.fcf_yield_ev)}</td>
                  <td>{pct(item.fcf_yield_vs_hurdle)}</td>
                  <td className="home-note">{item.note || "—"}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}
