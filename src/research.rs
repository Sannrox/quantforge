use crate::domain::{Financials, Ohlcv, newest_first};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Multiples {
    pub pe: Option<f64>,
    pub p_fcf: Option<f64>,
    pub p_ocf: Option<f64>,
    pub earnings_yield: Option<f64>,
    pub fcf_yield: Option<f64>,
    pub ocf_yield: Option<f64>,
    pub net_cash: Option<f64>,
    pub enterprise_value: Option<f64>,
    pub ev_fcf: Option<f64>,
    pub fcf_yield_ev: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Point {
    pub date: String,
    pub value: Option<f64>,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub yoy: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SeriesSet {
    pub revenue: Vec<Point>,
    pub ebitda: Vec<Point>,
    pub fcf: Vec<Point>,
    pub ocf: Vec<Point>,
    pub eps: Vec<Point>,
    pub gross_margin: Vec<Point>,
    pub operating_margin: Vec<Point>,
    pub net_margin: Vec<Point>,
    pub fcf_margin: Vec<Point>,
    pub pe: Vec<Point>,
    pub p_fcf: Vec<Point>,
    pub p_ocf: Vec<Point>,
    pub shares: Vec<Point>,
    pub fcf_ps: Vec<Point>,
    pub fcf_conversion: Vec<Point>,
    pub reinvestment: Vec<Point>,
    pub roic: Vec<Point>,
    pub net_cash: Vec<Point>,
    pub interest_coverage: Vec<Point>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Snapshot {
    pub years: u32,
    pub revenue_cagr: Option<f64>,
    pub revenue_cagr_5y: Option<f64>,
    pub revenue_cagr_fade: Option<f64>,
    pub fcf_cagr: Option<f64>,
    pub fcf_cagr_5y: Option<f64>,
    pub fcf_cagr_fade: Option<f64>,
    pub fcf_ps_cagr: Option<f64>,
    pub fcf_ps_cagr_5y: Option<f64>,
    pub fcf_ps_cagr_fade: Option<f64>,
    pub eps_cagr: Option<f64>,
    pub eps_cagr_5y: Option<f64>,
    pub eps_cagr_fade: Option<f64>,
    pub pe_median: Option<f64>,
    pub pe_p25: Option<f64>,
    pub pe_p75: Option<f64>,
    pub pe_percentile: Option<f64>,
    pub p_fcf_median: Option<f64>,
    pub p_fcf_p25: Option<f64>,
    pub p_fcf_p75: Option<f64>,
    pub p_fcf_percentile: Option<f64>,
    pub pe_high: Option<f64>,
    pub pe_vs_high: Option<f64>,
    pub p_fcf_high: Option<f64>,
    pub p_fcf_vs_high: Option<f64>,
    pub pe_vs_median: Option<f64>,
    pub p_fcf_vs_median: Option<f64>,
    pub p_ocf_median: Option<f64>,
    pub p_ocf_p25: Option<f64>,
    pub p_ocf_p75: Option<f64>,
    pub p_ocf_percentile: Option<f64>,
    pub p_ocf_high: Option<f64>,
    pub p_ocf_vs_high: Option<f64>,
    pub p_ocf_vs_median: Option<f64>,
    pub fcf_yield_median: Option<f64>,
    pub fcf_yield_vs_median: Option<f64>,
    pub ocf_yield_median: Option<f64>,
    pub ocf_yield_vs_median: Option<f64>,
    pub earnings_yield_median: Option<f64>,
    pub earnings_yield_vs_median: Option<f64>,
    pub fcf_power: Option<f64>,
    pub ocf_power: Option<f64>,
    pub years_to_median_p_fcf: Option<f64>,
    pub years_to_median_pe: Option<f64>,
    pub years_to_median_p_ocf: Option<f64>,
    pub share_change: Option<f64>,
    pub share_cagr: Option<f64>,
    pub fcf_conversion: Option<f64>,
    pub fcf_conversion_median: Option<f64>,
    pub fcf_conversion_vs_median: Option<f64>,
    pub fcf_conversion_3y: Option<f64>,
    pub fcf_conversion_3y_vs_median: Option<f64>,
    pub gross_margin: Option<f64>,
    pub gross_margin_median: Option<f64>,
    pub gross_margin_vs_median: Option<f64>,
    pub net_margin: Option<f64>,
    pub net_margin_median: Option<f64>,
    pub net_margin_vs_median: Option<f64>,
    pub operating_margin: Option<f64>,
    pub operating_margin_median: Option<f64>,
    pub operating_margin_vs_median: Option<f64>,
    pub operating_margin_3y: Option<f64>,
    pub operating_margin_3y_vs_median: Option<f64>,
    pub fcf_margin: Option<f64>,
    pub fcf_margin_median: Option<f64>,
    pub fcf_margin_vs_median: Option<f64>,
    pub fcf_margin_3y: Option<f64>,
    pub fcf_margin_3y_vs_median: Option<f64>,
    pub fcf_margin_iqr: Option<f64>,
    pub reinvestment: Option<f64>,
    pub reinvestment_median: Option<f64>,
    pub reinvestment_vs_median: Option<f64>,
    pub fcf_positive_years: u32,
    pub fcf_years: u32,
    pub fcf_up_years: u32,
    pub fcf_pairs: u32,
    pub revenue_up_years: u32,
    pub revenue_pairs: u32,
    pub ocf_up_years: u32,
    pub ocf_pairs: u32,
    pub roic: Option<f64>,
    pub roic_median: Option<f64>,
    pub roic_vs_median: Option<f64>,
    pub roic_3y: Option<f64>,
    pub roic_3y_vs_median: Option<f64>,
    pub fcf_yield_ev_median: Option<f64>,
    pub fcf_yield_ev_vs_median: Option<f64>,
    pub interest_coverage: Option<f64>,
    pub interest_coverage_median: Option<f64>,
    pub interest_coverage_vs_median: Option<f64>,
    pub interest_coverage_3y: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct StatementRow {
    pub period_end: String,
    pub fiscal_period: String,
    pub currency: String,
    pub revenue: Option<f64>,
    pub ebitda: Option<f64>,
    pub gross_profit: Option<f64>,
    pub operating_income: Option<f64>,
    pub net_income: Option<f64>,
    pub operating_cash_flow: Option<f64>,
    pub free_cash_flow: Option<f64>,
    pub eps: Option<f64>,
    pub shares_outstanding: Option<f64>,
    pub gross_margin: Option<f64>,
    pub operating_margin: Option<f64>,
    pub net_margin: Option<f64>,
    pub fcf_margin: Option<f64>,
    pub revenue_yoy: Option<f64>,
    pub net_income_yoy: Option<f64>,
    pub free_cash_flow_yoy: Option<f64>,
    pub operating_cash_flow_yoy: Option<f64>,
    pub eps_yoy: Option<f64>,
    pub shares_yoy: Option<f64>,
    pub operating_margin_yoy: Option<f64>,
    pub net_margin_yoy: Option<f64>,
    pub fcf_margin_yoy: Option<f64>,
    pub cash: Option<f64>,
    pub debt: Option<f64>,
    pub interest_expense: Option<f64>,
    pub interest_coverage: Option<f64>,
}

pub fn ratio(numer: Option<f64>, denom: Option<f64>) -> Option<f64> {
    match (numer, denom) {
        (Some(numer), Some(denom)) if denom.abs() > 0.0 => Some(numer / denom),
        _ => None,
    }
}

pub fn current_multiples(
    price: f64,
    market_cap: Option<f64>,
    annual: &[Financials],
    quarterly: &[Financials],
) -> Multiples {
    let annual = newest_first(annual);
    let quarterly = newest_first(quarterly);
    let ttm = ttm_row(&quarterly).or_else(|| annual.first().cloned());
    let Some(row) = ttm else {
        return Multiples {
            pe: None,
            p_fcf: None,
            p_ocf: None,
            earnings_yield: None,
            fcf_yield: None,
            ocf_yield: None,
            net_cash: None,
            enterprise_value: None,
            ev_fcf: None,
            fcf_yield_ev: None,
        };
    };
    let shares = row.shares_outstanding.filter(|shares| *shares > 0.0);
    let fcf_ps = match (row.free_cash_flow, shares) {
        (Some(fcf), Some(shares)) => Some(fcf / shares),
        _ => None,
    };
    let ocf_ps = match (row.operating_cash_flow, shares) {
        (Some(ocf), Some(shares)) => Some(ocf / shares),
        _ => None,
    };
    let pe = ratio(Some(price), row.eps);
    let p_fcf = ratio(Some(price), fcf_ps);
    let p_ocf = ratio(Some(price), ocf_ps);
    let balance = latest_balance(&annual, &quarterly).unwrap_or(&row);
    let net_cash = net_cash(balance);
    let enterprise_value = enterprise_value(equity_value(price, market_cap, shares), net_cash);
    Multiples {
        pe,
        p_fcf,
        p_ocf,
        earnings_yield: yield_of(pe),
        fcf_yield: yield_of(p_fcf),
        ocf_yield: yield_of(p_ocf),
        net_cash,
        enterprise_value,
        ev_fcf: ratio(enterprise_value, row.free_cash_flow),
        fcf_yield_ev: yield_of(ratio(enterprise_value, row.free_cash_flow)),
    }
}

pub fn statements(rows: &[Financials]) -> Vec<StatementRow> {
    let rows = newest_first(rows);
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            let prior = rows[index + 1..]
                .iter()
                .find(|prior| prior.fiscal_period == row.fiscal_period);
            StatementRow {
                period_end: row.period_end.clone(),
                fiscal_period: row.fiscal_period.clone(),
                currency: row.currency.clone(),
                revenue: row.revenue,
                ebitda: row.ebitda,
                gross_profit: row.gross_profit,
                operating_income: row.operating_income,
                net_income: row.net_income,
                operating_cash_flow: row.operating_cash_flow,
                free_cash_flow: row.free_cash_flow,
                eps: row.eps,
                shares_outstanding: row.shares_outstanding,
                gross_margin: ratio(row.gross_profit, row.revenue),
                operating_margin: ratio(row.operating_income, row.revenue),
                net_margin: ratio(row.net_income, row.revenue),
                fcf_margin: ratio(row.free_cash_flow, row.revenue),
                revenue_yoy: yoy(row.revenue, prior.and_then(|row| row.revenue)),
                net_income_yoy: yoy(row.net_income, prior.and_then(|row| row.net_income)),
                free_cash_flow_yoy: yoy(
                    row.free_cash_flow,
                    prior.and_then(|row| row.free_cash_flow),
                ),
                operating_cash_flow_yoy: yoy(
                    row.operating_cash_flow,
                    prior.and_then(|row| row.operating_cash_flow),
                ),
                eps_yoy: yoy(row.eps, prior.and_then(|row| row.eps)),
                shares_yoy: yoy(
                    row.shares_outstanding,
                    prior.and_then(|row| row.shares_outstanding),
                ),
                operating_margin_yoy: fade(
                    ratio(row.operating_income, row.revenue),
                    prior.and_then(|row| ratio(row.operating_income, row.revenue)),
                ),
                net_margin_yoy: fade(
                    ratio(row.net_income, row.revenue),
                    prior.and_then(|row| ratio(row.net_income, row.revenue)),
                ),
                fcf_margin_yoy: fade(
                    ratio(row.free_cash_flow, row.revenue),
                    prior.and_then(|row| ratio(row.free_cash_flow, row.revenue)),
                ),
                cash: row.cash,
                debt: row.debt,
                interest_expense: row.interest_expense,
                interest_coverage: interest_coverage(row),
            }
        })
        .collect()
}

pub fn quarterly_series(
    quarterly: &[Financials],
    annual: &[Financials],
    prices: &[Ohlcv],
) -> SeriesSet {
    let prints = series(quarterly, prices);
    let ttm = rolling_ttm(quarterly);
    let mut tape = chronological_prices(prices);
    tape.extend(price_hints(annual));
    let valued = series(&ttm, &tape);
    SeriesSet {
        pe: valued.pe,
        p_fcf: valued.p_fcf,
        p_ocf: valued.p_ocf,
        roic: valued.roic,
        interest_coverage: valued.interest_coverage,
        ..prints
    }
}

pub fn series(annual: &[Financials], prices: &[Ohlcv]) -> SeriesSet {
    let annual = newest_first(annual);
    let prices = chronological_prices(prices);
    let chronological: Vec<&Financials> = annual.iter().rev().collect();
    SeriesSet {
        revenue: map_points(&chronological, |row| row.revenue),
        ebitda: map_points(&chronological, |row| row.ebitda),
        fcf: map_points(&chronological, |row| row.free_cash_flow),
        ocf: map_points(&chronological, |row| row.operating_cash_flow),
        eps: map_points(&chronological, |row| row.eps),
        gross_margin: map_points(&chronological, |row| ratio(row.gross_profit, row.revenue)),
        operating_margin: map_points(&chronological, |row| {
            ratio(row.operating_income, row.revenue)
        }),
        net_margin: map_points(&chronological, |row| ratio(row.net_income, row.revenue)),
        fcf_margin: map_points(&chronological, |row| ratio(row.free_cash_flow, row.revenue)),
        pe: valuation_points(&chronological, &prices, |row| row.eps),
        p_fcf: valuation_points(&chronological, &prices, |row| {
            per_share(row.free_cash_flow, row.shares_outstanding)
        }),
        p_ocf: valuation_points(&chronological, &prices, |row| {
            per_share(row.operating_cash_flow, row.shares_outstanding)
        }),
        shares: map_points(&chronological, |row| row.shares_outstanding),
        fcf_ps: map_points(&chronological, |row| {
            per_share(row.free_cash_flow, row.shares_outstanding)
        }),
        fcf_conversion: map_points(&chronological, |row| {
            ratio(row.free_cash_flow, row.net_income)
        }),
        reinvestment: map_points(&chronological, |row| {
            reinvestment_rate(row.operating_cash_flow, row.free_cash_flow, row.revenue)
        }),
        roic: roic_points(&chronological),
        net_cash: map_points(&chronological, net_cash),
        interest_coverage: map_points(&chronological, interest_coverage),
    }
}

pub fn snapshot(
    annual: &[Financials],
    quarterly: &[Financials],
    prices: &[Ohlcv],
    multiples: &Multiples,
) -> Snapshot {
    let annual = newest_first(annual);
    let quarterly = newest_first(quarterly);
    let prices = chronological_prices(prices);
    let set = series(&annual, &prices);
    let years = year_span(
        annual.last().map(|row| row.period_end.as_str()),
        annual.first().map(|row| row.period_end.as_str()),
    )
    .unwrap_or(0);
    let pe_median = median_of(&set.pe);
    let p_fcf_median = median_of(&set.p_fcf);
    let p_ocf_median = median_of(&set.p_ocf);
    let pe_high = max_of(&set.pe);
    let p_fcf_high = max_of(&set.p_fcf);
    let p_ocf_high = max_of(&set.p_ocf);
    let fcf_yields = invert_positive(&set.p_fcf);
    let fcf_yield_median = median_of(&fcf_yields);
    let ocf_yields = invert_positive(&set.p_ocf);
    let ocf_yield_median = median_of(&ocf_yields);
    let earnings_yields = invert_positive(&set.pe);
    let earnings_yield_median = median_of(&earnings_yields);
    let ocf_ps: Vec<Point> = set
        .ocf
        .iter()
        .zip(set.shares.iter())
        .map(|(ocf, shares)| Point {
            date: ocf.date.clone(),
            value: per_share(ocf.value, shares.value),
            label: ocf.label.clone(),
            yoy: None,
        })
        .collect();
    let ocf_ps_cagr = cagr_of(&ocf_ps);
    let ocf_ps_cagr_5y = cagr_last_years(&ocf_ps, 5);
    let (fcf_positive_years, fcf_years) = positive_count(&set.fcf);
    let (fcf_up_years, fcf_pairs) = up_years(&set.fcf);
    let (ocf_up_years, ocf_pairs) = up_years(&set.ocf);
    let (revenue_up_years, revenue_pairs) = up_years(&set.revenue);
    let ttm = ttm_row(&quarterly).or_else(|| annual.first().cloned());
    let fcf_margins: Vec<Point> = set
        .revenue
        .iter()
        .zip(set.fcf.iter())
        .map(|(revenue, fcf)| Point {
            date: revenue.date.clone(),
            value: ratio(fcf.value, revenue.value),
            label: revenue.label.clone(),
            yoy: None,
        })
        .collect();
    let operating_margin = ttm
        .as_ref()
        .and_then(|row| ratio(row.operating_income, row.revenue))
        .or_else(|| latest_of(&set.operating_margin));
    let operating_margin_median = median_of(&set.operating_margin);
    let fcf_margin = ttm
        .as_ref()
        .and_then(|row| ratio(row.free_cash_flow, row.revenue))
        .or_else(|| latest_of(&fcf_margins));
    let fcf_margin_median = median_of(&fcf_margins);
    let fcf_conversion = ttm
        .as_ref()
        .and_then(|row| ratio(row.free_cash_flow, row.net_income))
        .or_else(|| latest_of(&set.fcf_conversion));
    let fcf_conversion_median = median_of(&set.fcf_conversion);
    let gross_margin = ttm
        .as_ref()
        .and_then(|row| ratio(row.gross_profit, row.revenue))
        .or_else(|| latest_of(&set.gross_margin));
    let gross_margin_median = median_of(&set.gross_margin);
    let net_margin = ttm
        .as_ref()
        .and_then(|row| ratio(row.net_income, row.revenue))
        .or_else(|| latest_of(&set.net_margin));
    let net_margin_median = median_of(&set.net_margin);
    let fcf_conversion_3y = mean_last(&set.fcf_conversion, 3);
    let share_change = relative_change(&set.shares);
    let reinvestment = ttm
        .as_ref()
        .and_then(|row| reinvestment_rate(row.operating_cash_flow, row.free_cash_flow, row.revenue))
        .or_else(|| latest_of(&set.reinvestment));
    let reinvestment_median = median_of(&set.reinvestment);
    let operating_margin_3y = mean_last(&set.operating_margin, 3);
    let fcf_margin_3y = mean_last(&set.fcf_margin, 3);
    let roic = ttm
        .as_ref()
        .and_then(|row| roic(row, annual.get(1)))
        .or_else(|| latest_of(&set.roic));
    let roic_median = median_of(&set.roic);
    let roic_3y = mean_last(&set.roic, 3);
    let ev_fcf_yields = invert_positive(&historical_ev_fcf(&annual, &prices));
    let fcf_yield_ev_median = median_of(&ev_fcf_yields);
    let interest_coverage = ttm
        .as_ref()
        .and_then(interest_coverage)
        .or_else(|| latest_of(&set.interest_coverage));
    let interest_coverage_median = median_of(&set.interest_coverage);
    let interest_coverage_3y = mean_last(&set.interest_coverage, 3);
    let revenue_cagr = cagr_of(&set.revenue);
    let revenue_cagr_5y = cagr_last_years(&set.revenue, 5);
    let fcf_cagr = cagr_of(&set.fcf);
    let fcf_cagr_5y = cagr_last_years(&set.fcf, 5);
    let fcf_ps_cagr = cagr_of(&set.fcf_ps);
    let fcf_ps_cagr_5y = cagr_last_years(&set.fcf_ps, 5);
    let eps_cagr = cagr_of(&set.eps);
    let eps_cagr_5y = cagr_last_years(&set.eps, 5);
    Snapshot {
        years,
        revenue_cagr,
        revenue_cagr_5y,
        revenue_cagr_fade: fade(revenue_cagr_5y, revenue_cagr),
        fcf_cagr,
        fcf_cagr_5y,
        fcf_cagr_fade: fade(fcf_cagr_5y, fcf_cagr),
        fcf_ps_cagr,
        fcf_ps_cagr_5y,
        fcf_ps_cagr_fade: fade(fcf_ps_cagr_5y, fcf_ps_cagr),
        eps_cagr,
        eps_cagr_5y,
        eps_cagr_fade: fade(eps_cagr_5y, eps_cagr),
        pe_median,
        pe_p25: quantile_of(&set.pe, 0.25),
        pe_p75: quantile_of(&set.pe, 0.75),
        pe_percentile: percentile_of(multiples.pe, &set.pe),
        p_fcf_median,
        p_fcf_p25: quantile_of(&set.p_fcf, 0.25),
        p_fcf_p75: quantile_of(&set.p_fcf, 0.75),
        p_fcf_percentile: percentile_of(multiples.p_fcf, &set.p_fcf),
        pe_high,
        pe_vs_high: vs_median(multiples.pe, pe_high),
        p_fcf_high,
        p_fcf_vs_high: vs_median(multiples.p_fcf, p_fcf_high),
        pe_vs_median: vs_median(multiples.pe, pe_median),
        p_fcf_vs_median: vs_median(multiples.p_fcf, p_fcf_median),
        p_ocf_median,
        p_ocf_p25: quantile_of(&set.p_ocf, 0.25),
        p_ocf_p75: quantile_of(&set.p_ocf, 0.75),
        p_ocf_percentile: percentile_of(multiples.p_ocf, &set.p_ocf),
        p_ocf_high,
        p_ocf_vs_high: vs_median(multiples.p_ocf, p_ocf_high),
        p_ocf_vs_median: vs_median(multiples.p_ocf, p_ocf_median),
        fcf_yield_median,
        fcf_yield_vs_median: vs_median(multiples.fcf_yield, fcf_yield_median),
        ocf_yield_median,
        ocf_yield_vs_median: vs_median(multiples.ocf_yield, ocf_yield_median),
        earnings_yield_median,
        earnings_yield_vs_median: vs_median(multiples.earnings_yield, earnings_yield_median),
        fcf_power: sum_opt(multiples.fcf_yield, fcf_ps_cagr_5y.or(fcf_ps_cagr)),
        ocf_power: sum_opt(multiples.ocf_yield, ocf_ps_cagr_5y.or(ocf_ps_cagr)),
        years_to_median_p_fcf: years_to_median(
            multiples.p_fcf,
            p_fcf_median,
            fcf_ps_cagr_5y.or(fcf_ps_cagr),
        ),
        years_to_median_pe: years_to_median(multiples.pe, pe_median, eps_cagr_5y.or(eps_cagr)),
        years_to_median_p_ocf: years_to_median(
            multiples.p_ocf,
            p_ocf_median,
            ocf_ps_cagr_5y.or(ocf_ps_cagr),
        ),
        share_change,
        share_cagr: annualized(share_change, years),
        fcf_conversion,
        fcf_conversion_median,
        fcf_conversion_vs_median: vs_median(fcf_conversion, fcf_conversion_median),
        fcf_conversion_3y,
        fcf_conversion_3y_vs_median: vs_median(fcf_conversion_3y, fcf_conversion_median),
        gross_margin,
        gross_margin_median,
        gross_margin_vs_median: vs_median(gross_margin, gross_margin_median),
        net_margin,
        net_margin_median,
        net_margin_vs_median: vs_median(net_margin, net_margin_median),
        operating_margin,
        operating_margin_median,
        operating_margin_vs_median: vs_median(operating_margin, operating_margin_median),
        operating_margin_3y,
        operating_margin_3y_vs_median: vs_median(operating_margin_3y, operating_margin_median),
        fcf_margin,
        fcf_margin_median,
        fcf_margin_vs_median: vs_median(fcf_margin, fcf_margin_median),
        fcf_margin_3y,
        fcf_margin_3y_vs_median: vs_median(fcf_margin_3y, fcf_margin_median),
        fcf_margin_iqr: iqr_of(&set.fcf_margin),
        reinvestment,
        reinvestment_median,
        reinvestment_vs_median: vs_median(reinvestment, reinvestment_median),
        fcf_positive_years,
        fcf_years,
        fcf_up_years,
        fcf_pairs,
        revenue_up_years,
        revenue_pairs,
        ocf_up_years,
        ocf_pairs,
        roic,
        roic_median,
        roic_vs_median: vs_median(roic, roic_median),
        roic_3y,
        roic_3y_vs_median: vs_median(roic_3y, roic_median),
        fcf_yield_ev_median,
        fcf_yield_ev_vs_median: vs_median(multiples.fcf_yield_ev, fcf_yield_ev_median),
        interest_coverage,
        interest_coverage_median,
        interest_coverage_vs_median: vs_median(interest_coverage, interest_coverage_median),
        interest_coverage_3y,
    }
}

fn map_points(rows: &[&Financials], value: impl Fn(&Financials) -> Option<f64>) -> Vec<Point> {
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            let current = value(row);
            let prior = rows[..index]
                .iter()
                .rev()
                .find(|prior| prior.fiscal_period == row.fiscal_period)
                .and_then(|prior| value(prior));
            point(row, current, yoy(current, prior))
        })
        .collect()
}

fn valuation_points(
    rows: &[&Financials],
    prices: &[Ohlcv],
    per_share: impl Fn(&Financials) -> Option<f64>,
) -> Vec<Point> {
    rows.iter()
        .map(|row| {
            let price = row
                .year_end_price
                .or_else(|| year_end_price(prices, &row.period_end));
            point(row, ratio(price, per_share(row)), None)
        })
        .collect()
}

fn per_share(total: Option<f64>, shares: Option<f64>) -> Option<f64> {
    match (total, shares) {
        (Some(total), Some(shares)) if shares > 0.0 => Some(total / shares),
        _ => None,
    }
}

fn reinvestment_rate(
    operating_cash_flow: Option<f64>,
    free_cash_flow: Option<f64>,
    revenue: Option<f64>,
) -> Option<f64> {
    match (operating_cash_flow, free_cash_flow, revenue) {
        (Some(ocf), Some(fcf), Some(revenue)) if revenue.abs() > 0.0 => Some((ocf - fcf) / revenue),
        _ => None,
    }
}

fn interest_coverage(row: &Financials) -> Option<f64> {
    let ebit = row.operating_income.filter(|value| *value > 0.0)?;
    let interest = row.interest_expense.filter(|value| *value > 0.0)?;
    Some(ebit / interest)
}

fn net_cash(row: &Financials) -> Option<f64> {
    match (row.cash, row.debt) {
        (Some(cash), Some(debt)) => Some(cash - debt),
        (Some(cash), None) => Some(cash),
        (None, Some(debt)) => Some(-debt),
        (None, None) => None,
    }
}

fn latest_balance<'a>(
    annual: &'a [Financials],
    quarterly: &'a [Financials],
) -> Option<&'a Financials> {
    quarterly
        .iter()
        .find(|row| row.cash.is_some() || row.debt.is_some())
        .or_else(|| {
            annual
                .iter()
                .find(|row| row.cash.is_some() || row.debt.is_some())
        })
}

fn equity_value(price: f64, market_cap: Option<f64>, shares: Option<f64>) -> Option<f64> {
    market_cap
        .filter(|cap| *cap > 0.0)
        .or_else(|| match shares {
            Some(shares) if shares > 0.0 && price > 0.0 => Some(price * shares),
            _ => None,
        })
}

fn enterprise_value(equity: Option<f64>, net_cash: Option<f64>) -> Option<f64> {
    let ev = equity? - net_cash?;
    (ev > 0.0).then_some(ev)
}

fn tax_rate(row: &Financials) -> f64 {
    match (row.tax_expense, row.pretax_income) {
        (Some(tax), Some(pretax)) if pretax > 0.0 => {
            let rate = tax / pretax;
            if (0.0..=0.6).contains(&rate) {
                rate
            } else {
                0.21
            }
        }
        _ => 0.21,
    }
}

fn nopat(row: &Financials) -> Option<f64> {
    row.operating_income
        .map(|ebit| ebit * (1.0 - tax_rate(row)))
}

fn invested_capital(row: &Financials) -> Option<f64> {
    let equity = row.equity?;
    let capital = equity + row.debt.unwrap_or(0.0) - row.cash.unwrap_or(0.0);
    (capital > 0.0).then_some(capital)
}

fn roic(row: &Financials, prior: Option<&Financials>) -> Option<f64> {
    let nopat = nopat(row)?;
    let current = invested_capital(row)?;
    let capital = prior
        .and_then(invested_capital)
        .map(|previous| (previous + current) / 2.0)
        .unwrap_or(current);
    (capital > 0.0).then_some(nopat / capital)
}

fn roic_points(rows: &[&Financials]) -> Vec<Point> {
    rows.iter()
        .enumerate()
        .map(|(index, row)| {
            let prior = index.checked_sub(1).and_then(|i| rows.get(i)).copied();
            let current = roic(row, prior);
            let year_ago = rows[..index]
                .iter()
                .rev()
                .find(|candidate| candidate.fiscal_period == row.fiscal_period)
                .and_then(|candidate| {
                    let at = rows
                        .iter()
                        .position(|item| std::ptr::eq(*item, *candidate))?;
                    let before = at.checked_sub(1).and_then(|i| rows.get(i)).copied();
                    roic(candidate, before)
                });
            point(row, current, yoy(current, year_ago))
        })
        .collect()
}

fn historical_ev_fcf(rows: &[Financials], prices: &[Ohlcv]) -> Vec<Point> {
    rows.iter()
        .map(|row| {
            let price = row
                .year_end_price
                .or_else(|| year_end_price(prices, &row.period_end));
            let equity = match (price, row.shares_outstanding) {
                (Some(price), Some(shares)) if shares > 0.0 => Some(price * shares),
                _ => None,
            };
            point(
                row,
                ratio(enterprise_value(equity, net_cash(row)), row.free_cash_flow),
                None,
            )
        })
        .collect()
}

fn year_end_price(prices: &[Ohlcv], period_end: &str) -> Option<f64> {
    let year = period_end.get(0..4)?;
    prices
        .iter()
        .rev()
        .find(|bar| bar.date.starts_with(year) && bar.date.as_str() <= period_end)
        // Statements are split-adjusted; use adj_close when present so historical multiples stay aligned.
        .map(|bar| bar.adj_close.unwrap_or(bar.close))
}

fn yield_of(multiple: Option<f64>) -> Option<f64> {
    match multiple {
        Some(multiple) if multiple > 0.0 => Some(1.0 / multiple),
        _ => None,
    }
}

fn invert_positive(points: &[Point]) -> Vec<Point> {
    points
        .iter()
        .map(|point| Point {
            date: point.date.clone(),
            value: yield_of(point.value),
            label: point.label.clone(),
            yoy: None,
        })
        .collect()
}

fn max_of(points: &[Point]) -> Option<f64> {
    points
        .iter()
        .filter_map(|point| point.value)
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
}

fn cagr_of(points: &[Point]) -> Option<f64> {
    let (first, last) = ends(points)?;
    let years = year_span(Some(first.0), Some(last.0))?;
    if first.1 <= 0.0 || last.1 <= 0.0 {
        return None;
    }
    Some((last.1 / first.1).powf(1.0 / f64::from(years)) - 1.0)
}

fn cagr_last_years(points: &[Point], years: u32) -> Option<f64> {
    let last = points
        .iter()
        .rev()
        .find_map(|point| Some((point.date.as_str(), point.value?)))?;
    let last_year: u32 = last.0.get(0..4)?.parse().ok()?;
    let target = last_year.checked_sub(years)?;
    let first = points
        .iter()
        .filter_map(|point| {
            let value = point.value?;
            let year: u32 = point.date.get(0..4)?.parse().ok()?;
            if year.abs_diff(target) <= 1 {
                Some((point.date.as_str(), value, year))
            } else {
                None
            }
        })
        .min_by_key(|(_, _, year)| (year.abs_diff(target), *year))?;
    let span = last_year.checked_sub(first.2).filter(|span| *span > 0)?;
    if first.1 <= 0.0 || last.1 <= 0.0 {
        return None;
    }
    Some((last.1 / first.1).powf(1.0 / f64::from(span)) - 1.0)
}

fn relative_change(points: &[Point]) -> Option<f64> {
    let (first, last) = ends(points)?;
    if first.1.abs() <= 0.0 {
        return None;
    }
    Some((last.1 / first.1) - 1.0)
}

fn ends(points: &[Point]) -> Option<((&str, f64), (&str, f64))> {
    let first = points
        .iter()
        .find_map(|point| Some((point.date.as_str(), point.value?)))?;
    let last = points
        .iter()
        .rev()
        .find_map(|point| Some((point.date.as_str(), point.value?)))?;
    Some((first, last))
}

fn latest_of(points: &[Point]) -> Option<f64> {
    points.iter().rev().find_map(|point| point.value)
}

fn median_of(points: &[Point]) -> Option<f64> {
    quantile_of(points, 0.5)
}

fn sorted_values(points: &[Point]) -> Vec<f64> {
    let mut values: Vec<f64> = points.iter().filter_map(|point| point.value).collect();
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    values
}

fn quantile_of(points: &[Point], quantile: f64) -> Option<f64> {
    let values = sorted_values(points);
    if values.is_empty() {
        return None;
    }
    if values.len() == 1 {
        return Some(values[0]);
    }
    let index = quantile.clamp(0.0, 1.0) * (values.len() - 1) as f64;
    let lo = index.floor() as usize;
    let hi = index.ceil() as usize;
    let weight = index - lo as f64;
    Some(values[lo] * (1.0 - weight) + values[hi] * weight)
}

fn percentile_of(current: Option<f64>, points: &[Point]) -> Option<f64> {
    let current = current?;
    let values = sorted_values(points);
    if values.len() < 4 {
        return None;
    }
    let below = values.iter().filter(|value| **value < current).count();
    Some(below as f64 / (values.len() - 1) as f64)
}

fn positive_count(points: &[Point]) -> (u32, u32) {
    let values: Vec<f64> = points.iter().filter_map(|point| point.value).collect();
    let positive = values.iter().filter(|value| **value > 0.0).count();
    (positive as u32, values.len() as u32)
}

fn up_years(points: &[Point]) -> (u32, u32) {
    let values: Vec<f64> = points.iter().filter_map(|point| point.value).collect();
    if values.len() < 2 {
        return (0, 0);
    }
    let pairs = values.len() - 1;
    let up = values.windows(2).filter(|pair| pair[1] > pair[0]).count();
    (up as u32, pairs as u32)
}

fn annualized(total: Option<f64>, years: u32) -> Option<f64> {
    let total = total?;
    if years == 0 {
        return None;
    }
    let base = 1.0 + total;
    if base <= 0.0 {
        return None;
    }
    Some(base.powf(1.0 / f64::from(years)) - 1.0)
}

fn chronological_prices(prices: &[Ohlcv]) -> Vec<Ohlcv> {
    let mut prices = prices.to_vec();
    prices.sort_by(|left, right| left.date.cmp(&right.date));
    prices
}

fn yoy(current: Option<f64>, prior: Option<f64>) -> Option<f64> {
    match (current, prior) {
        (Some(current), Some(prior)) if prior.abs() > 0.0 => Some(current / prior - 1.0),
        _ => None,
    }
}

fn years_to_median(current: Option<f64>, median: Option<f64>, growth: Option<f64>) -> Option<f64> {
    let current = current?;
    let median = median?;
    if current <= 0.0 || median <= 0.0 {
        return None;
    }
    if current <= median {
        return Some(0.0);
    }
    let growth = growth?;
    if growth <= 0.0 {
        return None;
    }
    Some((current / median).ln() / (1.0 + growth).ln())
}

fn sum_opt(left: Option<f64>, right: Option<f64>) -> Option<f64> {
    match (left, right) {
        (Some(left), Some(right)) => Some(left + right),
        _ => None,
    }
}

fn iqr_of(points: &[Point]) -> Option<f64> {
    if sorted_values(points).len() < 4 {
        return None;
    }
    match (quantile_of(points, 0.25), quantile_of(points, 0.75)) {
        (Some(low), Some(high)) => Some(high - low),
        _ => None,
    }
}

fn fade(recent: Option<f64>, long: Option<f64>) -> Option<f64> {
    match (recent, long) {
        (Some(recent), Some(long)) => Some(recent - long),
        _ => None,
    }
}

fn mean_last(points: &[Point], n: usize) -> Option<f64> {
    if n == 0 {
        return None;
    }
    let values: Vec<f64> = points
        .iter()
        .rev()
        .filter_map(|point| point.value)
        .take(n)
        .collect();
    if values.len() < n {
        return None;
    }
    Some(values.iter().sum::<f64>() / n as f64)
}

fn vs_median(current: Option<f64>, median: Option<f64>) -> Option<f64> {
    match (current, median) {
        (Some(current), Some(median)) if median.abs() > 0.0 => Some((current / median) - 1.0),
        _ => None,
    }
}

fn year_span(start: Option<&str>, end: Option<&str>) -> Option<u32> {
    let start: u32 = start?.get(0..4)?.parse().ok()?;
    let end: u32 = end?.get(0..4)?.parse().ok()?;
    end.checked_sub(start).filter(|years| *years > 0)
}

fn ttm_row(quarterly: &[Financials]) -> Option<Financials> {
    if quarterly.len() < 4 {
        return None;
    }
    ttm_window(&quarterly[..4])
}

fn rolling_ttm(quarterly: &[Financials]) -> Vec<Financials> {
    let rows = newest_first(quarterly);
    if rows.len() < 4 {
        return Vec::new();
    }
    (0..=rows.len() - 4)
        .filter_map(|start| ttm_window(&rows[start..start + 4]))
        .collect()
}

fn ttm_window(newest_first: &[Financials]) -> Option<Financials> {
    if newest_first.len() < 4 {
        return None;
    }
    let last = &newest_first[0];
    let take = &newest_first[..4];
    let sum = |pick: fn(&Financials) -> Option<f64>| -> Option<f64> {
        let values: Vec<f64> = take.iter().filter_map(pick).collect();
        if values.len() == 4 {
            Some(values.iter().sum())
        } else {
            None
        }
    };
    Some(Financials {
        period_end: last.period_end.clone(),
        fiscal_period: "TTM".into(),
        currency: last.currency.clone(),
        revenue: sum(|row| row.revenue),
        ebitda: sum(|row| row.ebitda),
        gross_profit: sum(|row| row.gross_profit),
        operating_income: sum(|row| row.operating_income),
        net_income: sum(|row| row.net_income),
        operating_cash_flow: sum(|row| row.operating_cash_flow),
        free_cash_flow: sum(|row| row.free_cash_flow),
        eps: sum(|row| row.eps),
        shares_outstanding: last.shares_outstanding,
        year_end_price: last.year_end_price,
        cash: last.cash,
        debt: last.debt,
        equity: last.equity,
        pretax_income: sum(|row| row.pretax_income),
        tax_expense: sum(|row| row.tax_expense),
        interest_expense: sum(|row| row.interest_expense),
    })
}

fn price_hints(annual: &[Financials]) -> Vec<Ohlcv> {
    annual
        .iter()
        .filter_map(|row| {
            Some(Ohlcv {
                date: row.period_end.clone(),
                close: row.year_end_price?,
                adj_close: row.year_end_price,
            })
        })
        .collect()
}

pub fn price_series(prices: &[Ohlcv], annual: &[Financials]) -> Vec<Point> {
    let tape = chronological_prices(prices);
    if !tape.is_empty() {
        return tape
            .iter()
            .map(|bar| Point {
                date: bar.date.clone(),
                value: Some(bar.adj_close.unwrap_or(bar.close)),
                label: bar.date.get(0..4).unwrap_or(bar.date.as_str()).to_string(),
                yoy: None,
            })
            .collect();
    }
    let annual = newest_first(annual);
    let chronological: Vec<&Financials> = annual.iter().rev().collect();
    map_points(&chronological, |row| row.year_end_price)
}

fn point(row: &Financials, value: Option<f64>, yoy: Option<f64>) -> Point {
    Point {
        date: row.period_end.clone(),
        value,
        label: period_label(row),
        yoy,
    }
}

fn period_label(row: &Financials) -> String {
    let year2 = row.period_end.get(2..4).unwrap_or("");
    let year4 = row.period_end.get(0..4).unwrap_or(row.period_end.as_str());
    match row.fiscal_period.as_str() {
        "FY" => year4.to_string(),
        "TTM" => format!("TTM {year4}"),
        period if period.starts_with('Q') => format!("{period}'{year2}"),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn margins_and_multiples() {
        let annual = [Financials {
            period_end: "2024-12-31".into(),
            fiscal_period: "FY".into(),
            currency: "USD".into(),
            revenue: Some(100.0),
            ebitda: Some(30.0),
            gross_profit: Some(40.0),
            operating_income: Some(25.0),
            net_income: Some(20.0),
            operating_cash_flow: Some(28.0),
            free_cash_flow: Some(18.0),
            eps: Some(2.0),
            shares_outstanding: Some(10.0),
            year_end_price: Some(40.0),
            ..Financials::default()
        }];
        let multiples = current_multiples(40.0, None, &annual, &[]);
        assert_eq!(multiples.pe, Some(20.0));
        assert_eq!(multiples.p_fcf, Some(40.0 / 1.8));
        assert_eq!(multiples.earnings_yield, Some(0.05));
        assert!((multiples.fcf_yield.unwrap() - 1.8 / 40.0).abs() < 1e-9);
        assert!((multiples.ocf_yield.unwrap() - 2.8 / 40.0).abs() < 1e-9);
        let rows = statements(&annual);
        assert_eq!(rows[0].gross_margin, Some(0.4));
        assert_eq!(rows[0].fcf_margin, Some(0.18));
        let set = series(&annual, &[]);
        assert_eq!(set.pe[0].value, Some(20.0));
        assert_eq!(set.shares[0].value, Some(10.0));
        assert_eq!(set.ocf[0].value, Some(28.0));
        assert_eq!(set.ocf[0].value, Some(28.0));
        assert_eq!(set.fcf_ps[0].value, Some(1.8));
        assert_eq!(set.fcf_conversion[0].value, Some(0.9));
        assert_eq!(set.reinvestment[0].value, Some(0.1));
    }

    #[test]
    fn snapshot_cagr_median_and_share_change() {
        let annual = [
            Financials {
                period_end: "2024-12-31".into(),
                fiscal_period: "FY".into(),
                currency: "USD".into(),
                revenue: Some(200.0),
                ebitda: Some(60.0),
                gross_profit: Some(80.0),
                operating_income: Some(50.0),
                net_income: Some(40.0),
                operating_cash_flow: Some(55.0),
                free_cash_flow: Some(36.0),
                eps: Some(4.0),
                shares_outstanding: Some(8.0),
                year_end_price: Some(40.0),
                ..Financials::default()
            },
            Financials {
                period_end: "2014-12-31".into(),
                fiscal_period: "FY".into(),
                currency: "USD".into(),
                revenue: Some(100.0),
                ebitda: Some(30.0),
                gross_profit: Some(40.0),
                operating_income: Some(25.0),
                net_income: Some(20.0),
                operating_cash_flow: Some(28.0),
                free_cash_flow: Some(18.0),
                eps: Some(2.0),
                shares_outstanding: Some(10.0),
                year_end_price: Some(20.0),
                ..Financials::default()
            },
        ];
        let multiples = current_multiples(40.0, None, &annual, &[]);
        let snap = snapshot(&annual, &[], &[], &multiples);
        assert_eq!(snap.years, 10);
        assert!((snap.revenue_cagr.unwrap() - (2.0_f64.powf(0.1) - 1.0)).abs() < 1e-9);
        assert!((snap.share_change.unwrap() - (-0.2)).abs() < 1e-9);
        assert!((snap.share_cagr.unwrap() - (0.8_f64.powf(0.1) - 1.0)).abs() < 1e-9);
        assert_eq!(snap.net_margin, Some(0.2));
        assert_eq!(snap.net_margin_vs_median, Some(0.0));
        assert_eq!(snap.fcf_conversion_3y, None);
        assert_eq!(snap.pe_median, Some(10.0));
        assert_eq!(
            snap.p_ocf_median,
            Some((20.0 / 2.8 + 40.0 / (55.0 / 8.0)) / 2.0)
        );
        assert!((snap.pe_vs_median.unwrap() - 0.0).abs() < 1e-9);
        assert_eq!(snap.fcf_conversion, Some(0.9));
        assert_eq!(snap.fcf_conversion_median, Some(0.9));
        assert_eq!(snap.fcf_conversion_vs_median, Some(0.0));
        assert_eq!(snap.gross_margin, Some(0.4));
        assert_eq!(snap.gross_margin_vs_median, Some(0.0));
        assert_eq!(snap.operating_margin, Some(0.25));
        assert_eq!(snap.fcf_margin, Some(0.18));
        assert!((snap.fcf_ps_cagr.unwrap() - ((4.5 / 1.8_f64).powf(0.1) - 1.0)).abs() < 1e-9);
        assert_eq!(snap.fcf_margin_iqr, None);
        assert!((snap.fcf_power.unwrap() - (4.5 / 40.0 + snap.fcf_ps_cagr.unwrap())).abs() < 1e-9);
        let ocf_ps_cagr = (6.875_f64 / 2.8).powf(0.1) - 1.0;
        assert!((snap.ocf_power.unwrap() - (6.875 / 40.0 + ocf_ps_cagr)).abs() < 1e-9);
        assert!((snap.reinvestment.unwrap() - 0.095).abs() < 1e-9);
        assert_eq!(snap.revenue_cagr_5y, None);
        assert_eq!(snap.revenue_cagr_fade, None);
        assert_eq!(snap.operating_margin_3y, None);
        assert_eq!(snap.pe_percentile, None);
        assert_eq!(snap.fcf_positive_years, 2);
        assert_eq!(snap.fcf_years, 2);
        assert_eq!(snap.fcf_up_years, 1);
        assert_eq!(snap.fcf_pairs, 1);
        assert_eq!(snap.ocf_up_years, 1);
        assert_eq!(snap.ocf_pairs, 1);
        assert_eq!(snap.revenue_up_years, 1);
        assert_eq!(snap.revenue_pairs, 1);
        assert_eq!(snap.pe_p25, Some(10.0));
        assert_eq!(snap.pe_p75, Some(10.0));
        assert_eq!(snap.pe_high, Some(10.0));
        assert_eq!(snap.pe_vs_high, Some(0.0));
        assert_eq!(snap.years_to_median_p_fcf, Some(0.0));
        assert_eq!(snap.years_to_median_pe, Some(0.0));
        assert_eq!(snap.years_to_median_p_ocf, Some(0.0));
        assert!(
            (snap.ocf_yield_median.unwrap() - ((2.8 / 20.0) + (55.0 / 8.0 / 40.0)) / 2.0).abs()
                < 1e-9
        );
        let rows = statements(&annual);
        assert!((rows[0].revenue_yoy.unwrap() - 1.0).abs() < 1e-9);
        assert!((rows[0].net_income_yoy.unwrap() - 1.0).abs() < 1e-9);
        assert!((rows[0].free_cash_flow_yoy.unwrap() - 1.0).abs() < 1e-9);
        assert!((rows[0].operating_cash_flow_yoy.unwrap() - (55.0 / 28.0 - 1.0)).abs() < 1e-9);
        assert!((rows[0].eps_yoy.unwrap() - 1.0).abs() < 1e-9);
        assert!((rows[0].shares_yoy.unwrap() - (-0.2)).abs() < 1e-9);
        assert_eq!(rows[0].operating_margin_yoy, Some(0.0));
        assert_eq!(rows[0].net_margin_yoy, Some(0.0));
        assert_eq!(rows[0].fcf_margin_yoy, Some(0.0));
        assert_eq!(snap.earnings_yield_median, Some(0.1));
        assert_eq!(snap.earnings_yield_vs_median, Some(0.0));
        assert_eq!(rows[1].revenue_yoy, None);
        let reversed = [annual[1].clone(), annual[0].clone()];
        let reversed_rows = statements(&reversed);
        assert!(
            (reversed_rows[0].revenue_yoy.unwrap() - rows[0].revenue_yoy.unwrap()).abs() < 1e-9
        );
        let reversed_snap = snapshot(&reversed, &[], &[], &multiples);
        assert_eq!(reversed_snap.years, snap.years);
        assert!((reversed_snap.revenue_cagr.unwrap() - snap.revenue_cagr.unwrap()).abs() < 1e-9);
        assert!(
            (snap.fcf_yield_median.unwrap() - ((1.8 / 20.0) + (4.5 / 40.0)) / 2.0).abs() < 1e-9
        );
    }

    fn fy(end: &str, revenue: f64, fcf: f64, eps: f64, shares: f64, price: f64) -> Financials {
        Financials {
            period_end: end.into(),
            fiscal_period: "FY".into(),
            currency: "USD".into(),
            revenue: Some(revenue),
            ebitda: Some(revenue * 0.3),
            gross_profit: Some(revenue * 0.4),
            operating_income: Some(revenue * 0.25),
            net_income: Some(revenue * 0.2),
            operating_cash_flow: Some(fcf * 1.2),
            free_cash_flow: Some(fcf),
            eps: Some(eps),
            shares_outstanding: Some(shares),
            year_end_price: Some(price),
            ..Financials::default()
        }
    }

    #[test]
    fn snapshot_five_year_range_and_consistency() {
        let annual = [
            fy("2025-12-31", 200.0, 36.0, 4.0, 8.0, 40.0),
            fy("2024-12-31", 180.0, 32.0, 3.6, 8.2, 36.0),
            fy("2023-12-31", 160.0, 20.0, 4.0, 8.4, 24.0),
            fy("2022-12-31", 140.0, 16.0, 2.0, 8.6, 40.0),
            fy("2021-12-31", 120.0, 14.0, 3.0, 8.8, 30.0),
            fy("2020-12-31", 100.0, 12.0, 2.0, 9.0, 20.0),
        ];
        let multiples = current_multiples(40.0, None, &annual, &[]);
        let snap = snapshot(&annual, &[], &[], &multiples);
        assert_eq!(snap.years, 5);
        assert!((snap.revenue_cagr.unwrap() - (2.0_f64.powf(0.2) - 1.0)).abs() < 1e-9);
        assert!((snap.revenue_cagr_5y.unwrap() - (2.0_f64.powf(0.2) - 1.0)).abs() < 1e-9);
        assert!((snap.revenue_cagr_fade.unwrap()).abs() < 1e-9);
        assert_eq!(snap.operating_margin_3y, Some(0.25));
        assert!(
            (snap.fcf_conversion_3y.unwrap() - (0.9 + 32.0 / 36.0 + 20.0 / 32.0) / 3.0).abs()
                < 1e-9
        );
        assert_eq!(snap.net_margin, Some(0.2));
        assert_eq!(snap.operating_margin_3y_vs_median, Some(0.0));
        assert_eq!(snap.fcf_up_years, 5);
        assert_eq!(snap.fcf_pairs, 5);
        assert!(snap.fcf_margin_iqr.unwrap() > 0.0);
        assert!(snap.fcf_power.unwrap() > multiples.fcf_yield.unwrap());
        assert!(snap.ocf_power.unwrap() > multiples.ocf_yield.unwrap());
        assert_eq!(multiples.pe, Some(10.0));
        assert_eq!(multiples.earnings_yield, Some(0.1));
        // Historical P/E: 10, 10, 6, 20, 10, 10. Current 10 sits above only 6.
        assert!((snap.pe_percentile.unwrap() - 0.2).abs() < 1e-9);
        assert_eq!(snap.pe_p25, Some(10.0));
        assert_eq!(snap.pe_high, Some(20.0));
        assert_eq!(snap.pe_vs_high, Some(-0.5));
        assert_eq!(snap.fcf_positive_years, 6);
        assert_eq!(snap.fcf_years, 6);
        assert_eq!(snap.revenue_up_years, 5);
        assert_eq!(snap.revenue_pairs, 5);
    }

    #[test]
    fn snapshot_uses_price_tape_when_statements_omit_year_end() {
        let annual = [Financials {
            period_end: "2024-12-31".into(),
            fiscal_period: "FY".into(),
            currency: "USD".into(),
            revenue: Some(100.0),
            ebitda: Some(30.0),
            gross_profit: Some(40.0),
            operating_income: Some(25.0),
            net_income: Some(20.0),
            operating_cash_flow: Some(28.0),
            free_cash_flow: Some(18.0),
            eps: Some(2.0),
            shares_outstanding: Some(10.0),
            year_end_price: None,
            ..Financials::default()
        }];
        let prices = [Ohlcv {
            date: "2024-12-31".into(),
            close: 40.0,
            adj_close: None,
        }];
        let multiples = current_multiples(40.0, None, &annual, &[]);
        let snap = snapshot(&annual, &[], &prices, &multiples);
        assert_eq!(snap.pe_median, Some(20.0));
        assert_eq!(snap.pe_vs_median, Some(0.0));
    }

    #[test]
    fn statements_compare_quarters_to_year_ago() {
        let quarter = |end: &str, period: &str, revenue: f64| Financials {
            period_end: end.into(),
            fiscal_period: period.into(),
            currency: "USD".into(),
            revenue: Some(revenue),
            ebitda: None,
            gross_profit: None,
            operating_income: None,
            net_income: Some(revenue * 0.2),
            operating_cash_flow: None,
            free_cash_flow: Some(revenue * 0.18),
            eps: Some(revenue / 50.0),
            shares_outstanding: None,
            year_end_price: None,
            ..Financials::default()
        };
        let rows = statements(&[
            quarter("2025-12-31", "Q4", 110.0),
            quarter("2025-09-30", "Q3", 100.0),
            quarter("2025-03-31", "Q1", 80.0),
            quarter("2024-12-31", "Q4", 100.0),
        ]);
        assert!((rows[0].revenue_yoy.unwrap() - 0.1).abs() < 1e-9);
        assert!((rows[0].net_income_yoy.unwrap() - 0.1).abs() < 1e-9);
        assert!((rows[0].eps_yoy.unwrap() - 0.1).abs() < 1e-9);
        assert_eq!(rows[0].fcf_margin_yoy, Some(0.0));
        assert_eq!(rows[1].revenue_yoy, None);
        assert_eq!(rows[2].revenue_yoy, None);
    }

    fn qtr(end: &str, period: &str, revenue: f64, fcf: f64, eps: f64, shares: f64) -> Financials {
        Financials {
            period_end: end.into(),
            fiscal_period: period.into(),
            currency: "USD".into(),
            revenue: Some(revenue),
            ebitda: Some(revenue * 0.3),
            gross_profit: Some(revenue * 0.45),
            operating_income: Some(revenue * 0.25),
            net_income: Some(revenue * 0.2),
            operating_cash_flow: Some(fcf * 1.2),
            free_cash_flow: Some(fcf),
            eps: Some(eps),
            shares_outstanding: Some(shares),
            year_end_price: None,
            ..Financials::default()
        }
    }

    #[test]
    fn quarterly_series_prints_levels_and_ttm_multiples() {
        let quarterly = [
            qtr("2025-12-31", "Q4", 40.0, 8.0, 1.1, 10.0),
            qtr("2025-09-30", "Q3", 30.0, 6.0, 0.9, 10.0),
            qtr("2025-06-30", "Q2", 20.0, 4.0, 0.6, 10.0),
            qtr("2025-03-31", "Q1", 10.0, 2.0, 0.4, 10.0),
            qtr("2024-12-31", "Q4", 36.0, 7.0, 1.0, 10.0),
        ];
        let annual = [fy("2025-12-31", 100.0, 20.0, 3.0, 10.0, 60.0)];
        let set = quarterly_series(&quarterly, &annual, &[]);
        assert_eq!(set.revenue.last().and_then(|point| point.value), Some(40.0));
        assert_eq!(
            set.revenue.last().map(|point| point.label.as_str()),
            Some("Q4'25")
        );
        assert!(
            (set.revenue.last().and_then(|point| point.yoy).unwrap() - (40.0 / 36.0 - 1.0)).abs()
                < 1e-9
        );
        let prices = price_series(&[], &annual);
        assert_eq!(prices.last().and_then(|point| point.value), Some(60.0));
        assert_eq!(
            prices.last().map(|point| point.label.as_str()),
            Some("2025")
        );
        assert_eq!(set.fcf.last().and_then(|point| point.value), Some(8.0));
        assert_eq!(set.pe.len(), 2);
        assert_eq!(set.pe.last().and_then(|point| point.value), Some(20.0));
        assert_eq!(
            set.pe.last().map(|point| point.label.as_str()),
            Some("TTM 2025")
        );
        assert_eq!(set.p_fcf.last().and_then(|point| point.value), Some(30.0));
    }

    #[test]
    fn roic_and_enterprise_value_use_the_balance_sheet() {
        let prior = Financials {
            period_end: "2023-12-31".into(),
            fiscal_period: "FY".into(),
            currency: "USD".into(),
            operating_income: Some(25.0),
            pretax_income: Some(25.0),
            tax_expense: Some(5.25),
            free_cash_flow: Some(18.0),
            eps: Some(2.0),
            shares_outstanding: Some(10.0),
            year_end_price: Some(20.0),
            cash: Some(10.0),
            debt: Some(5.0),
            equity: Some(40.0),
            interest_expense: Some(1.0),
            ..Financials::default()
        };
        let latest = Financials {
            period_end: "2024-12-31".into(),
            fiscal_period: "FY".into(),
            currency: "USD".into(),
            operating_income: Some(50.0),
            pretax_income: Some(50.0),
            tax_expense: Some(10.5),
            free_cash_flow: Some(36.0),
            eps: Some(4.0),
            shares_outstanding: Some(10.0),
            year_end_price: Some(40.0),
            cash: Some(20.0),
            debt: Some(8.0),
            equity: Some(80.0),
            interest_expense: Some(2.0),
            ..Financials::default()
        };
        let annual = [latest, prior];
        let multiples = current_multiples(40.0, Some(400.0), &annual, &[]);
        assert_eq!(multiples.net_cash, Some(12.0));
        assert_eq!(multiples.enterprise_value, Some(388.0));
        assert!((multiples.fcf_yield_ev.unwrap() - 36.0 / 388.0).abs() < 1e-9);
        let set = series(&annual, &[]);
        assert!((set.roic[0].value.unwrap() - (25.0 * 0.79 / 35.0)).abs() < 1e-9);
        assert!((set.roic[1].value.unwrap() - (50.0 * 0.79 / 51.5)).abs() < 1e-9);
        assert_eq!(set.net_cash[1].value, Some(12.0));
        let snap = snapshot(&annual, &[], &[], &multiples);
        assert!((snap.roic.unwrap() - (50.0 * 0.79 / 51.5)).abs() < 1e-9);
        assert!(snap.roic_median.is_some());
        assert!(snap.fcf_yield_ev_median.is_some());
        assert_eq!(snap.interest_coverage, Some(25.0));
        assert_eq!(set.interest_coverage[1].value, Some(25.0));
        assert_eq!(set.interest_coverage[0].value, Some(25.0));
    }
}
