use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::num::NonZeroUsize;
use std::path::Path;

use serde::Deserialize;
use speculant_core::economy::Real;

/// Файл порогов в каталоге данных; игра его не читает.
pub const BALANCE_FILE: &str = "balance.toml";

/// Пороги баланса, торговли и покрытия из `data/balance.toml`, снятые калибровкой от измеренной
/// базы прогона; `[volume]` того же файла читает `cargo xtask content-count`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct BalanceFile {
    pub balance: BalanceRules,
    pub trade: TradeRules,
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

/// Торговля окупается, но не ломает экономику: выгода лучшего рейса и перевес перекупщика над
/// ботом, который не торгует с толком.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TradeRules {
    /// Кошелёк, с которым меряется рейс, в рублях 1984 года.
    pub purse: Real,
    /// Медиана выгоды лучшего рейса по неделям эпохи — прибыль к кошельку.
    pub trip_return: Bounds,
    /// Во сколько раз медиана капитала арбитражника к концу эпохи выше медианы случайного бота —
    /// по эпохам.
    pub arbitrage_over_random: BTreeMap<String, Bounds>,
}

/// Пределы величины, оба включительно.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub from: f64,
    pub to: f64,
}

impl Bounds {
    pub fn contains(self, value: f64) -> bool {
        self.from <= value && value <= self.to
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoverageRules {
    /// Жизней в прогоне динамического покрытия валидатора.
    pub lives: NonZeroUsize,
    /// Доля решений, в которых бот прогона покрытия выбирает наугад, не кончая жизнь, — чтобы
    /// заглянуть в продолжения, которые манера бота сама не выберет; от 0 до 1.
    pub curiosity: f64,
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
        let file: Self = toml::from_str(text).map_err(|error| BalanceFileError::Format {
            reason: error.to_string(),
        })?;
        if !(0.0..=1.0).contains(&file.coverage.curiosity) {
            return Err(BalanceFileError::Format {
                reason: format!(
                    "[coverage] curiosity {}: нужна доля от 0 до 1",
                    file.coverage.curiosity
                ),
            });
        }
        Ok(file)
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
