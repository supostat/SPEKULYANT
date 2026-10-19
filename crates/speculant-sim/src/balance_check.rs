//! Сверка прогона с порогами `[balance]`: беды и тишина каждой эпохи и стратегии, к её концу
//! ломающие экономику.

use std::fmt;

use crate::{BalanceRules, EpochReport, Report, Strategy};

/// Что сверять: одну эпоху или все.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckScope {
    Epoch(String),
    All,
}

impl CheckScope {
    const ALL: &str = "all";

    pub fn parse(text: &str) -> Self {
        if text == Self::ALL {
            Self::All
        } else {
            Self::Epoch(text.to_owned())
        }
    }
}

/// Порог, который прогон нарушил.
#[derive(Clone, Debug, PartialEq)]
pub enum Violation {
    Mortality {
        epoch: String,
        measured: f64,
        limit: f64,
    },
    /// В эпохе жили, а событий не было (`measured` — `None`) или они были слишком редки.
    Quiet {
        epoch: String,
        measured: Option<f64>,
        limit: f64,
    },
    Breaking {
        epoch: String,
        strategy: Strategy,
        measured: f64,
        limit: f64,
    },
}

/// Сверить нельзя: эпохи нет среди эпох прогона или для неё нет порога.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CheckError {
    UnknownEpoch {
        epoch: String,
        known: Vec<String>,
    },
    MissingThreshold {
        epoch: String,
        section: &'static str,
    },
}

const MORTALITY: &str = "[balance.mortality]";
const QUIET_DAYS: &str = "[balance.quiet_days]";

pub fn check(
    report: &Report,
    rules: &BalanceRules,
    scope: &CheckScope,
) -> Result<Vec<Violation>, CheckError> {
    let epochs: Vec<&EpochReport> = match scope {
        CheckScope::All => report.epochs.iter().collect(),
        CheckScope::Epoch(id) => {
            let epoch = report
                .epochs
                .iter()
                .find(|epoch| &epoch.epoch == id)
                .ok_or_else(|| CheckError::UnknownEpoch {
                    epoch: id.clone(),
                    known: report
                        .epochs
                        .iter()
                        .map(|epoch| epoch.epoch.clone())
                        .collect(),
                })?;
            vec![epoch]
        }
    };
    let mut violations = Vec::new();
    for epoch in epochs {
        violations.extend(check_epoch(epoch, rules)?);
    }
    Ok(violations)
}

fn check_epoch(epoch: &EpochReport, rules: &BalanceRules) -> Result<Vec<Violation>, CheckError> {
    let threshold = |section: &'static str,
                     thresholds: &std::collections::BTreeMap<String, f64>| {
        thresholds
            .get(&epoch.epoch)
            .copied()
            .ok_or_else(|| CheckError::MissingThreshold {
                epoch: epoch.epoch.clone(),
                section,
            })
    };
    let mortality_limit = threshold(MORTALITY, &rules.mortality)?;
    let quiet_limit = threshold(QUIET_DAYS, &rules.quiet_days)?;
    let mut violations = Vec::new();
    if let Some(measured) = epoch.mortality.filter(|rate| *rate > mortality_limit) {
        violations.push(Violation::Mortality {
            epoch: epoch.epoch.clone(),
            measured,
            limit: mortality_limit,
        });
    }
    let lived = epoch.life_years > 0.0;
    let too_quiet = epoch.quiet_days.is_none_or(|days| days > quiet_limit);
    if lived && too_quiet {
        violations.push(Violation::Quiet {
            epoch: epoch.epoch.clone(),
            measured: epoch.quiet_days,
            limit: quiet_limit,
        });
    }
    let strategies = epoch.capital.iter().flat_map(|capital| &capital.strategies);
    violations.extend(
        strategies
            .filter_map(|strategy| Some((strategy.strategy, strategy.medians?)))
            .filter(|(_, medians)| *medians > rules.breaking_medians)
            .map(|(strategy, measured)| Violation::Breaking {
                epoch: epoch.epoch.clone(),
                strategy,
                measured,
                limit: rules.breaking_medians,
            }),
    );
    Ok(violations)
}

impl fmt::Display for Violation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Mortality {
                epoch,
                measured,
                limit,
            } => write!(
                formatter,
                "{epoch}: бед {measured:.4} на жизне-год, порог {limit:.4} {MORTALITY}"
            ),
            Self::Quiet {
                epoch,
                measured: Some(days),
                limit,
            } => write!(
                formatter,
                "{epoch}: событие раз в {days:.1} дн., порог {limit:.1} {QUIET_DAYS}"
            ),
            Self::Quiet {
                epoch,
                measured: None,
                limit,
            } => write!(
                formatter,
                "{epoch}: событий нет вовсе, порог — раз в {limit:.1} дн. {QUIET_DAYS}"
            ),
            Self::Breaking {
                epoch,
                strategy,
                measured,
                limit,
            } => write!(
                formatter,
                "{epoch}: медиана капитала {strategy} к концу эпохи — {measured:.2} общих медиан, \
                 порог {limit:.2} [balance] breaking_medians"
            ),
        }
    }
}

impl fmt::Display for CheckError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownEpoch { epoch, known } => write!(
                formatter,
                "эпохи {epoch} нет среди эпох прогона: {} или all",
                known.join(", ")
            ),
            Self::MissingThreshold { epoch, section } => {
                write!(formatter, "для эпохи {epoch} нет порога в {section}")
            }
        }
    }
}

impl std::error::Error for CheckError {}
