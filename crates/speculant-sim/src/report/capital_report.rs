use serde::Serialize;
use speculant_core::economy::Real;

use crate::Strategy;

const LOW_SHARE: f64 = 0.1;
const MIDDLE_SHARE: f64 = 0.5;
const HIGH_SHARE: f64 = 0.9;

/// Капитал жизней в рублях 1984 года на один момент — конец эпохи или конец жизни: медиана всех
/// и разброс каждой стратегии.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct CapitalReport {
    pub median: Real,
    pub strategies: Vec<StrategyCapital>,
}

/// Капитал стратегии: десятый, пятидесятый и девяностый процентили и во сколько раз её медиана
/// выше общей — стратегия далеко выше всех ломает экономику.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StrategyCapital {
    pub strategy: Strategy,
    pub lives: usize,
    pub low: Real,
    pub median: Real,
    pub high: Real,
    /// `None`, если общая медиана не выше нуля.
    pub medians: Option<f64>,
}

impl CapitalReport {
    /// По капиталу каждой жизни со стратегией; `None`, если жизней нет.
    pub(crate) fn build(capitals: &[(Strategy, Real)]) -> Option<Self> {
        let median = ranked(capitals.iter().map(|(_, capital)| *capital), MIDDLE_SHARE)?;
        let strategies = Strategy::ALL
            .into_iter()
            .filter_map(|strategy| {
                let own: Vec<Real> = capitals
                    .iter()
                    .filter(|(owner, _)| *owner == strategy)
                    .map(|(_, capital)| *capital)
                    .collect();
                let own_median = ranked(own.iter().copied(), MIDDLE_SHARE)?;
                Some(StrategyCapital {
                    strategy,
                    lives: own.len(),
                    low: ranked(own.iter().copied(), LOW_SHARE)?,
                    median: own_median,
                    high: ranked(own.iter().copied(), HIGH_SHARE)?,
                    medians: median
                        .is_positive()
                        .then(|| own_median.kopecks() as f64 / median.kopecks() as f64),
                })
            })
            .collect();
        Some(Self { median, strategies })
    }
}

/// Капитал по ближайшему рангу: доля `share` жизней не богаче его; `None`, если жизней нет.
fn ranked(capitals: impl Iterator<Item = Real>, share: f64) -> Option<Real> {
    let mut kopecks: Vec<i64> = capitals.map(Real::kopecks).collect();
    if kopecks.is_empty() {
        return None;
    }
    kopecks.sort_unstable();
    let rank = ((share * kopecks.len() as f64).ceil() as usize).clamp(1, kopecks.len());
    Some(Real::from_kopecks(kopecks[rank - 1]))
}
