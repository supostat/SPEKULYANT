use serde::{Deserialize, Serialize};

use crate::content::CityId;
use crate::economy::Real;
use crate::time::GameDate;

/// Цена штуки в рублях 1984 года — где и когда.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceNote {
    pub price: Real,
    pub date: GameDate,
    pub city: CityId,
}
