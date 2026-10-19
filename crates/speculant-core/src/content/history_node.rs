use serde::Deserialize;

use super::{HistoryOutcome, ScaleRange};
use crate::time::GameDate;

/// Исторический узел (§3.6) из `data/history/<узел>.toml`: окно дат внутри одной эпохи — день в нём
/// выбирает сид, — разброс масштаба и альтернативные исходы с весами.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryNode {
    pub title: String,
    pub window: HistoryWindow,
    #[serde(default)]
    pub scale: ScaleRange,
    pub outcomes: Vec<HistoryOutcome>,
}

/// Окно дат узла: `from` входит, `until` — уже нет.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryWindow {
    pub from: GameDate,
    pub until: GameDate,
}
