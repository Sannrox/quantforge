export type WatchItem = {
  ticker: string;
  name: string | null;
  provider: string | null;
  price: number | null;
  currency: string | null;
  pe: number | null;
  p_fcf: number | null;
  p_ocf: number | null;
  pe_vs_median: number | null;
  p_fcf_vs_median: number | null;
  p_ocf_vs_median: number | null;
  pe_percentile: number | null;
  revenue_cagr: number | null;
  revenue_cagr_5y: number | null;
  revenue_cagr_fade: number | null;
  fcf_yield: number | null;
  fcf_yield_ev: number | null;
  fcf_yield_vs_median: number | null;
  fcf_conversion: number | null;
  fcf_yield_vs_hurdle: number | null;
  fcf_power_vs_hurdle: number | null;
  ocf_power_vs_hurdle: number | null;
  years_to_median_p_fcf: number | null;
  years_to_median_pe: number | null;
  years_to_median_p_ocf: number | null;
  upside: number | null;
  note: string;
};

export type Point = { date: string; value: number | null; label: string; yoy?: number | null };

export type SeriesSet = {
  revenue: Point[];
  ebitda: Point[];
  fcf: Point[];
  ocf: Point[];
  eps: Point[];
  gross_margin: Point[];
  operating_margin: Point[];
  net_margin: Point[];
  fcf_margin: Point[];
  pe: Point[];
  p_fcf: Point[];
  p_ocf: Point[];
  shares: Point[];
  fcf_ps: Point[];
    fcf_conversion: Point[];
    reinvestment: Point[];
    roic: Point[];
    net_cash: Point[];
};

export type StatementRow = {
  period_end: string;
  fiscal_period: string;
  currency: string;
  revenue: number | null;
  ebitda: number | null;
  gross_profit: number | null;
  operating_income: number | null;
  net_income: number | null;
  operating_cash_flow: number | null;
  free_cash_flow: number | null;
  eps: number | null;
  shares_outstanding: number | null;
  gross_margin: number | null;
  operating_margin: number | null;
  net_margin: number | null;
  fcf_margin: number | null;
  revenue_yoy: number | null;
  net_income_yoy: number | null;
  free_cash_flow_yoy: number | null;
  operating_cash_flow_yoy: number | null;
  eps_yoy: number | null;
  shares_yoy: number | null;
  operating_margin_yoy: number | null;
  net_margin_yoy: number | null;
  fcf_margin_yoy: number | null;
};

export type Company = {
  ticker: string;
  name: string;
  sector: string;
  currency: string;
  price: number;
  market_cap: number | null;
  provider: string;
  active_provider: string;
  fetched_at: string;
  multiples: {
    pe: number | null;
    p_fcf: number | null;
    p_ocf: number | null;
    earnings_yield: number | null;
    fcf_yield: number | null;
    ocf_yield: number | null;
    net_cash: number | null;
    enterprise_value: number | null;
    ev_fcf: number | null;
    fcf_yield_ev: number | null;
  };
  annual: StatementRow[];
  quarterly: StatementRow[];
  snapshot: {
    years: number;
    revenue_cagr: number | null;
    revenue_cagr_5y: number | null;
    revenue_cagr_fade: number | null;
    fcf_cagr: number | null;
    fcf_cagr_5y: number | null;
    fcf_cagr_fade: number | null;
    fcf_ps_cagr: number | null;
    fcf_ps_cagr_5y: number | null;
    fcf_ps_cagr_fade: number | null;
    eps_cagr: number | null;
    eps_cagr_5y: number | null;
    eps_cagr_fade: number | null;
    pe_median: number | null;
    pe_p25: number | null;
    pe_p75: number | null;
    pe_percentile: number | null;
    p_fcf_median: number | null;
    p_fcf_p25: number | null;
    p_fcf_p75: number | null;
    p_fcf_percentile: number | null;
    pe_high: number | null;
    pe_vs_high: number | null;
    p_fcf_high: number | null;
    p_fcf_vs_high: number | null;
    pe_vs_median: number | null;
    p_fcf_vs_median: number | null;
    p_ocf_median: number | null;
    p_ocf_p25: number | null;
    p_ocf_p75: number | null;
    p_ocf_percentile: number | null;
    p_ocf_high: number | null;
    p_ocf_vs_high: number | null;
    p_ocf_vs_median: number | null;
    fcf_yield_median: number | null;
    fcf_yield_vs_median: number | null;
    ocf_yield_median: number | null;
    ocf_yield_vs_median: number | null;
    earnings_yield_median: number | null;
    earnings_yield_vs_median: number | null;
    fcf_power: number | null;
    ocf_power: number | null;
    years_to_median_p_fcf: number | null;
    years_to_median_pe: number | null;
    years_to_median_p_ocf: number | null;
    share_change: number | null;
    share_cagr: number | null;
    fcf_conversion: number | null;
    fcf_conversion_median: number | null;
    fcf_conversion_vs_median: number | null;
    fcf_conversion_3y: number | null;
    fcf_conversion_3y_vs_median: number | null;
    gross_margin: number | null;
    gross_margin_median: number | null;
    gross_margin_vs_median: number | null;
    net_margin: number | null;
    net_margin_median: number | null;
    net_margin_vs_median: number | null;
    operating_margin: number | null;
    operating_margin_median: number | null;
    operating_margin_vs_median: number | null;
    operating_margin_3y: number | null;
    operating_margin_3y_vs_median: number | null;
    fcf_margin: number | null;
    fcf_margin_median: number | null;
    fcf_margin_vs_median: number | null;
    fcf_margin_3y: number | null;
    fcf_margin_3y_vs_median: number | null;
    fcf_margin_iqr: number | null;
    reinvestment: number | null;
    reinvestment_median: number | null;
    reinvestment_vs_median: number | null;
    fcf_positive_years: number;
    fcf_years: number;
    fcf_up_years: number;
    fcf_pairs: number;
    revenue_up_years: number;
    revenue_pairs: number;
    ocf_up_years: number;
    ocf_pairs: number;
    roic: number | null;
    roic_median: number | null;
    roic_vs_median: number | null;
    roic_3y: number | null;
    roic_3y_vs_median: number | null;
    fcf_yield_ev_median: number | null;
    fcf_yield_ev_vs_median: number | null;
  };
  series: SeriesSet;
  quarterly_series: SeriesSet;
  price_series: Point[];
  sensitivity: {
    growths: number[];
    returns: number[];
    cells: { growth: number; desired_return: number; fair_value: number | null }[];
  } | null;
  dcf: {
    fair_value: number;
    stream_value: number;
    net_cash_per_share: number | null;
    price: number;
    upside: number | null;
    seed_per_share: number;
    seed_kind: string;
    years: number;
    growth: number;
    desired_return: number;
  } | null;
  assumptions: { growth: number; desired_return: number };
  fcf_yield_vs_hurdle: number | null;
  fcf_yield_ev_vs_hurdle: number | null;
  fcf_power_vs_hurdle: number | null;
  ocf_power_vs_hurdle: number | null;
  note: string;
};

export type Settings = {
  provider: string;
  providers: string[];
  has_fmp_key: boolean;
};

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(path, {
    ...init,
    headers: {
      accept: "application/json",
      ...(init?.body ? { "content-type": "application/json" } : {}),
      ...init?.headers,
    },
  });
  const body = await response.json().catch(() => ({}));
  if (!response.ok) {
    throw new Error(typeof body.error === "string" ? body.error : response.statusText);
  }
  return body as T;
}

export const api = {
  watchlist: () => request<WatchItem[]>("/api/watchlist"),
  addWatch: (ticker: string) =>
    request<WatchItem[]>("/api/watchlist", { method: "POST", body: JSON.stringify({ ticker }) }),
  removeWatch: (ticker: string) => request<WatchItem[]>(`/api/watchlist/${ticker}`, { method: "DELETE" }),
  company: (ticker: string) => request<Company>(`/api/companies/${ticker}`),
  refresh: (ticker: string) => request<Company>(`/api/companies/${ticker}/refresh`, { method: "POST" }),
  saveDcf: (ticker: string, growth: number, desired_return: number) =>
    request<Company>(`/api/companies/${ticker}/dcf`, {
      method: "PUT",
      body: JSON.stringify({ growth, desired_return }),
    }),
  saveNote: (ticker: string, body: string) =>
    request<Company>(`/api/companies/${ticker}/notes`, {
      method: "PUT",
      body: JSON.stringify({ body }),
    }),
  settings: () => request<Settings>("/api/settings"),
  saveSettings: (body: { provider?: string; fmp_key?: string }) =>
    request<Settings>("/api/settings", { method: "PUT", body: JSON.stringify(body) }),
};
