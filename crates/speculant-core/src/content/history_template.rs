use serde::Deserialize;

use super::{HistoryOutcome, ScaleRange};

/// Шаблон генератора песочницы (решение 15.9) из `data/history/post2004/<шаблон>.toml`: вес среди
/// шаблонов, срок в днях, раньше которого он не повторяется, разброс масштаба и исходы с весами.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryTemplate {
    pub title: String,
    pub weight: u32,
    pub rest_days: u32,
    #[serde(default)]
    pub scale: ScaleRange,
    pub outcomes: Vec<HistoryOutcome>,
}
