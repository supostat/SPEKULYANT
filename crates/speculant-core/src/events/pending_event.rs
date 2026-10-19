use serde::{Deserialize, Serialize};

use super::{Resolution, Substitutions};
use crate::business::BusinessId;
use crate::content::{EventId, NpcId};

/// Событие, остановившее время: ждёт выбора, а после выбора — пока игрок прочтёт исход. Если в
/// событии участвует соперник, здесь — тот, кого выбрал сид; если это происшествие с делом —
/// само дело. Подстановки сняты в момент, когда событие случилось: исход читается с теми же
/// именами, даже если управляющий ушёл, а дело сгорело.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PendingEvent {
    pub event: EventId,
    pub rival: Option<NpcId>,
    pub business: Option<BusinessId>,
    pub substitutions: Substitutions,
    pub resolution: Option<Resolution>,
}
