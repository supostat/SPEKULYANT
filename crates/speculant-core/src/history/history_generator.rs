use serde::Deserialize;

use crate::time::GameDate;

/// Генератор песочницы из `data/history/generator.toml` (решение 15.9): с какого дня история идёт
/// из шаблонов, до какого горизонта раскладываются узлы и сколько дней между ними.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryGenerator {
    pub from: GameDate,
    pub until: GameDate,
    pub gap_days: DayRange,
}

/// Число дней от `from` до `to` включительно.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DayRange {
    pub from: u32,
    pub to: u32,
}
