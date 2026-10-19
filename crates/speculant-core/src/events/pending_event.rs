use super::Resolution;
use crate::content::{EventId, NpcId};

/// Событие, остановившее время: ждёт выбора, а после выбора — пока игрок прочтёт исход. Если в
/// событии участвует соперник, здесь — тот, кого выбрал сид.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PendingEvent {
    pub event: EventId,
    pub rival: Option<NpcId>,
    pub resolution: Option<Resolution>,
}
