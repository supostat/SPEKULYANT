use serde::{Deserialize, Serialize};

use crate::content::{BiographyId, EndingId};
use crate::economy::Real;
use crate::time::GameDate;

/// Строка таблицы рекордов (решение 15.10): кто, с какой биографией, с каким капиталом в рублях
/// 1984 года, когда и чем кончил.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Record {
    pub name: String,
    pub biography: BiographyId,
    pub capital: Real,
    pub ended: GameDate,
    pub ending: EndingId,
}
