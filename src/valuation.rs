use crate::domain::{DcfAssumptions, Financials, newest_first};

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct DcfResult {
    pub fair_value: f64,
    pub stream_value: f64,
    pub net_cash_per_share: Option<f64>,
    pub price: f64,
    pub upside: Option<f64>,
    pub seed_per_share: f64,
    pub seed_kind: &'static str,
    pub years: u32,
    pub growth: f64,
    pub desired_return: f64,
}

pub fn ttm_seed(annual: &[Financials], quarterly: &[Financials]) -> Option<(f64, &'static str)> {
    ttm_seed_parts(annual, quarterly).map(|(seed, kind, _shares)| (seed, kind))
}

pub fn ttm_share_count(annual: &[Financials], quarterly: &[Financials]) -> Option<f64> {
    ttm_seed_parts(annual, quarterly).map(|(_seed, _kind, shares)| shares)
}

fn ttm_seed_parts(
    annual: &[Financials],
    quarterly: &[Financials],
) -> Option<(f64, &'static str, f64)> {
    let per_share = |rows: &[Financials], take: usize| -> Option<(f64, &'static str, f64)> {
        let slice: Vec<&Financials> = rows.iter().take(take).collect();
        if slice.len() < take {
            return None;
        }
        let shares = slice
            .iter()
            .find_map(|row| row.shares_outstanding)
            .filter(|shares| *shares > 0.0)?;
        let fcf: Vec<f64> = slice.iter().filter_map(|row| row.free_cash_flow).collect();
        if fcf.len() == take {
            let fcf: f64 = fcf.iter().sum();
            if fcf > 0.0 {
                return Some((fcf / shares, "fcf", shares));
            }
        }
        let eps: Vec<f64> = slice.iter().filter_map(|row| row.eps).collect();
        if take == 1 {
            return slice
                .first()
                .and_then(|row| row.eps)
                .filter(|eps| *eps > 0.0)
                .map(|eps| (eps, "eps", shares));
        }
        if eps.len() == take {
            let eps: f64 = eps.iter().sum();
            if eps > 0.0 {
                return Some((eps, "eps", shares));
            }
        }
        None
    };

    let quarterly = newest_first(quarterly);
    let annual = newest_first(annual);
    per_share(&quarterly, 4).or_else(|| per_share(&annual, 1))
}

pub fn project(
    assumptions: &DcfAssumptions,
    seed_per_share: f64,
    seed_kind: &'static str,
    price: f64,
    net_cash_per_share: Option<f64>,
) -> DcfResult {
    let years = 10_u32;
    let growth = assumptions.growth;
    let rate = assumptions.desired_return;
    let mut stream_value = 0.0;
    let mut cash = seed_per_share;
    for year in 1..=years {
        cash *= 1.0 + growth;
        stream_value += cash / (1.0 + rate).powi(year as i32);
    }
    let terminal = if rate > growth {
        cash * (1.0 + growth) / (rate - growth)
    } else {
        cash / rate.max(0.01)
    };
    stream_value += terminal / (1.0 + rate).powi(years as i32);
    let fair_value = stream_value + net_cash_per_share.unwrap_or(0.0);
    let upside = if price > 0.0 {
        Some((fair_value - price) / price)
    } else {
        None
    };
    DcfResult {
        fair_value,
        stream_value,
        net_cash_per_share,
        price,
        upside,
        seed_per_share,
        seed_kind,
        years,
        growth,
        desired_return: rate,
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct SensitivityCell {
    pub growth: f64,
    pub desired_return: f64,
    pub fair_value: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Sensitivity {
    pub growths: Vec<f64>,
    pub returns: Vec<f64>,
    pub cells: Vec<SensitivityCell>,
}

pub fn sensitivity(
    seed_per_share: f64,
    seed_kind: &'static str,
    price: f64,
    base: &DcfAssumptions,
    net_cash_per_share: Option<f64>,
) -> Sensitivity {
    let growths = nearby(base.growth, 0.0, 1.0);
    let returns = nearby(base.desired_return, 0.01, 1.0);
    let mut cells = Vec::new();
    for desired_return in &returns {
        for growth in &growths {
            let fair_value = if *desired_return > *growth {
                Some(
                    project(
                        &DcfAssumptions {
                            growth: *growth,
                            desired_return: *desired_return,
                        },
                        seed_per_share,
                        seed_kind,
                        price,
                        net_cash_per_share,
                    )
                    .fair_value,
                )
            } else {
                None
            };
            cells.push(SensitivityCell {
                growth: *growth,
                desired_return: *desired_return,
                fair_value,
            });
        }
    }
    Sensitivity {
        growths,
        returns,
        cells,
    }
}

fn nearby(center: f64, min: f64, max: f64) -> Vec<f64> {
    let mut values = vec![center - 0.02, center, center + 0.02];
    values.retain(|value| *value >= min && *value <= max);
    values.sort_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal));
    values.dedup_by(|left, right| (*left - *right).abs() < 1e-12);
    values
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::Financials;

    fn row(fcf: f64, shares: f64, eps: f64) -> Financials {
        Financials {
            period_end: "2024-12-31".into(),
            fiscal_period: "FY".into(),
            currency: "USD".into(),
            free_cash_flow: Some(fcf),
            eps: Some(eps),
            shares_outstanding: Some(shares),
            ..Financials::default()
        }
    }

    #[test]
    fn seeds_from_annual_fcf_per_share() {
        let (seed, kind) = ttm_seed(&[row(200.0, 100.0, 1.0)], &[]).expect("seed");
        assert_eq!(kind, "fcf");
        assert!((seed - 2.0).abs() < 1e-9);
    }

    #[test]
    fn negative_fcf_falls_back_to_positive_eps() {
        let (seed, kind) = ttm_seed(&[row(-50.0, 100.0, 2.0)], &[]).expect("eps seed");
        assert_eq!(kind, "eps");
        assert!((seed - 2.0).abs() < 1e-9);
    }

    #[test]
    fn incomplete_quarterly_does_not_seed_as_ttm() {
        let quarters = [
            row(10.0, 100.0, 1.0),
            row(10.0, 100.0, 1.0),
            row(10.0, 100.0, 1.0),
            Financials {
                free_cash_flow: None,
                eps: None,
                ..row(0.0, 100.0, 0.0)
            },
        ];
        let (seed, kind) = ttm_seed(&[row(200.0, 100.0, 1.0)], &quarters).expect("annual fallback");
        assert_eq!(kind, "fcf");
        assert!((seed - 2.0).abs() < 1e-9);
    }

    #[test]
    fn dcf_is_above_seed_when_growth_beats_nothing() {
        let result = project(
            &DcfAssumptions {
                growth: 0.0,
                desired_return: 0.10,
            },
            1.0,
            "fcf",
            10.0,
            None,
        );
        assert!(result.fair_value > 0.0);
        assert_eq!(result.fair_value, result.stream_value);
        assert_eq!(result.net_cash_per_share, None);
        assert_eq!(result.years, 10);
    }

    #[test]
    fn net_cash_raises_equity_fair_value() {
        let assumptions = DcfAssumptions {
            growth: 0.08,
            desired_return: 0.12,
        };
        let without = project(&assumptions, 10.0, "fcf", 142.5, None);
        let cash = 15_000_000_000.0 / 1_993_000_000.0;
        let with = project(&assumptions, 10.0, "fcf", 142.5, Some(cash));
        assert!((with.stream_value - without.stream_value).abs() < 1e-9);
        assert!((with.fair_value - (without.fair_value + cash)).abs() < 1e-9);
        assert!((with.net_cash_per_share.unwrap() - cash).abs() < 1e-9);
        assert!(with.upside.unwrap() > without.upside.unwrap());
    }

    #[test]
    fn sensitivity_skips_growth_at_or_above_return() {
        let grid = sensitivity(
            1.0,
            "fcf",
            10.0,
            &DcfAssumptions {
                growth: 0.08,
                desired_return: 0.10,
            },
            None,
        );
        assert!(grid.growths.contains(&0.08));
        assert!(grid.returns.contains(&0.10));
        let invalid = grid.cells.iter().find(|cell| {
            (cell.growth - 0.10).abs() < 1e-12 && (cell.desired_return - 0.08).abs() < 1e-12
        });
        assert!(invalid.is_none() || invalid.unwrap().fair_value.is_none());
        let base = grid
            .cells
            .iter()
            .find(|cell| {
                (cell.growth - 0.08).abs() < 1e-12 && (cell.desired_return - 0.10).abs() < 1e-12
            })
            .expect("base cell");
        assert!(base.fair_value.unwrap() > 0.0);
    }

    #[test]
    fn sensitivity_cells_include_net_cash() {
        let assumptions = DcfAssumptions {
            growth: 0.08,
            desired_return: 0.10,
        };
        let cash = 5.0;
        let grid = sensitivity(1.0, "fcf", 10.0, &assumptions, Some(cash));
        let expected = project(&assumptions, 1.0, "fcf", 10.0, Some(cash)).fair_value;
        let base = grid
            .cells
            .iter()
            .find(|cell| {
                (cell.growth - 0.08).abs() < 1e-12 && (cell.desired_return - 0.10).abs() < 1e-12
            })
            .expect("base cell");
        assert!((base.fair_value.unwrap() - expected).abs() < 1e-9);
    }
}
