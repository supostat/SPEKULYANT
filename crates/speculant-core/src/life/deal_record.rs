use serde::{Deserialize, Serialize};

use crate::content::GoodId;
use crate::economy::Real;
use crate::time::GameDate;

/// Сделка для итогов жизни: какой товар, когда продан и сколько принёс сверх цены покупки в
/// рублях 1984 года; убыток — меньше нуля.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DealRecord {
    pub good: GoodId,
    pub date: GameDate,
    pub profit: Real,
}
