use crate::content::EventId;
use crate::time::GameDate;

/// Отложенное продолжение (§10.1): событие придёт в день `due`, если его условия ещё будут выполнены.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScheduledEvent {
    pub event: EventId,
    pub due: GameDate,
}
