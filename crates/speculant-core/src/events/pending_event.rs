use super::Resolution;
use crate::content::EventId;

/// Событие, остановившее время: ждёт выбора, а после выбора — пока игрок прочтёт исход.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingEvent {
    pub event: EventId,
    pub resolution: Option<Resolution>,
}
