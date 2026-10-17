use std::fmt;

use serde::Deserialize;

use super::{ClockSettings, Epoch, EpochTable, GameDate, MAX_SPEED};

/// Часы и календарь эпох из `data/epochs/time.toml`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeRules {
    pub clock: ClockSettings,
    pub epochs: EpochTable,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeFile {
    clock: ClockFile,
    epochs: Vec<EpochFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ClockFile {
    day_milliseconds: Vec<u32>,
    fast_forward_day_milliseconds: u32,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct EpochFile {
    id: String,
    title: String,
    starts: String,
}

impl TimeRules {
    pub fn from_toml(text: &str) -> Result<Self, TimeRulesError> {
        let file: TimeFile =
            toml::from_str(text).map_err(|error| TimeRulesError::Format(error.to_string()))?;
        let speeds: [u32; MAX_SPEED as usize] =
            file.clock
                .day_milliseconds
                .try_into()
                .map_err(|speeds: Vec<u32>| TimeRulesError::SpeedCount {
                    found: speeds.len(),
                })?;
        let clock = ClockSettings::new(speeds, file.clock.fast_forward_day_milliseconds)?;
        let epochs = file
            .epochs
            .into_iter()
            .map(|epoch| {
                let starts = GameDate::parse_iso(&epoch.starts).ok_or_else(|| {
                    TimeRulesError::EpochDate {
                        id: epoch.id.clone(),
                        text: epoch.starts.clone(),
                    }
                })?;
                Ok(Epoch {
                    id: epoch.id,
                    title: epoch.title,
                    starts,
                })
            })
            .collect::<Result<Vec<_>, TimeRulesError>>()?;
        Ok(Self {
            clock,
            epochs: EpochTable::new(epochs)?,
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TimeRulesError {
    Format(String),
    SpeedCount { found: usize },
    SpeedsNotAccelerating,
    FastForwardNotFastest,
    NoEpochs,
    FirstEpochStart { id: String },
    EpochOrder { id: String },
    DuplicateEpoch { id: String },
    EpochDate { id: String, text: String },
}

impl fmt::Display for TimeRulesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Format(problem) => write!(formatter, "time.toml не разбирается: {problem}"),
            Self::SpeedCount { found } => {
                write!(formatter, "скоростей {found}, а нужно {MAX_SPEED}")
            }
            Self::SpeedsNotAccelerating => write!(
                formatter,
                "каждая следующая скорость должна давать более короткий день"
            ),
            Self::FastForwardNotFastest => write!(
                formatter,
                "день перемотки должен быть короче дня самой быстрой скорости"
            ),
            Self::NoEpochs => write!(formatter, "в календаре нет ни одной эпохи"),
            Self::FirstEpochStart { id } => {
                write!(formatter, "первая эпоха {id} должна начинаться 1984-01-01")
            }
            Self::EpochOrder { id } => {
                write!(formatter, "эпоха {id} начинается не позже предыдущей")
            }
            Self::DuplicateEpoch { id } => write!(formatter, "эпоха {id} описана дважды"),
            Self::EpochDate { id, text } => write!(
                formatter,
                "у эпохи {id} дата начала «{text}» не в форме ГГГГ-ММ-ДД"
            ),
        }
    }
}

impl std::error::Error for TimeRulesError {}
