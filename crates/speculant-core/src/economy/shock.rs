use serde::{Deserialize, Serialize};

use crate::content::{CityId, GoodId};
use crate::time::GameDate;

/// Шок цены (§5.2 п. 4): указ, кризис или слух умножает цену товара в `factor` раз — в одном городе
/// или везде (`city = None`), до даты `until` или навсегда (`until = None`).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Shock {
    pub good: GoodId,
    pub city: Option<CityId>,
    pub factor: f64,
    pub until: Option<GameDate>,
}

impl Shock {
    pub fn applies_to(&self, good: &GoodId, city: &CityId) -> bool {
        &self.good == good && self.city.as_ref().is_none_or(|shocked| shocked == city)
    }

    pub fn is_over_on(&self, date: GameDate) -> bool {
        self.until.is_some_and(|until| date >= until)
    }
}
