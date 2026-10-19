use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;

use serde::Deserialize;

/// Файл порогов в каталоге данных; игра его не читает.
pub const BALANCE_FILE: &str = "balance.toml";

/// Пороги баланса и покрытия из `data/balance.toml`, снятые калибровкой от измеренной базы
/// прогона; `[volume]` того же файла читает `cargo xtask content-count`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BalanceFile {
    pub balance: BalanceRules,
    pub coverage: CoverageRules,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BalanceRules {
    /// Сид прогона `cargo xtask balance-check` — того, что CI гоняет на каждом пуше.
    pub seed: u64,
    /// Жизней в этом прогоне.
    pub lives: NonZeroUsize,
    /// Медиана капитала стратегии не выше стольких общих медиан — иначе стратегия ломает
    /// экономику.
    pub breaking_medians: f64,
    /// Бед на жизне-год по эпохам — не больше.
    pub mortality: BTreeMap<String, f64>,
    /// Дней жизни на одно событие по эпохам — не больше.
    pub quiet_days: BTreeMap<String, f64>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRules {
    /// Жизней в прогоне динамического покрытия валидатора.
    pub lives: NonZeroUsize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum BalanceFileError {
    Unreadable { path: String, reason: String },
    Format { reason: String },
}

impl BalanceFile {
    pub fn read(data: &Path) -> Result<Self, BalanceFileError> {
        let path = data.join(BALANCE_FILE);
        let text = fs::read_to_string(&path).map_err(|error| BalanceFileError::Unreadable {
            path: path.display().to_string(),
            reason: error.to_string(),
        })?;
        Self::parse(&text)
    }

    pub fn parse(text: &str) -> Result<Self, BalanceFileError> {
        toml::from_str(text).map_err(|error| BalanceFileError::Format {
            reason: error.to_string(),
        })
    }
}

impl fmt::Display for BalanceFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unreadable { path, reason } => write!(formatter, "не читается {path}: {reason}"),
            Self::Format { reason } => write!(formatter, "{BALANCE_FILE} не разбирается: {reason}"),
        }
    }
}

impl std::error::Error for BalanceFileError {}
