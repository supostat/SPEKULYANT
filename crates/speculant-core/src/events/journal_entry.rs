use serde::{Deserialize, Serialize};

use crate::content::EventId;
use crate::time::GameDate;

/// Запись журнала жизни: какое событие случилось в какой день и какой выбор сделан (номер с нуля).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JournalEntry {
    pub date: GameDate,
    pub event: EventId,
    pub choice: Option<usize>,
}
