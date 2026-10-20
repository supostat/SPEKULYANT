use std::collections::BTreeMap;

use serde::Deserialize;

use super::CategoryId;
use crate::time::GameDate;

/// Категория без множителя продаётся по кривой товара.
const NEUTRAL_MULTIPLIER: f64 = 1.0;

/// Город (§4): толкучка, множители цен по категориям товаров и доступность по датам.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct City {
    pub name: String,
    pub region: String,
    /// Где торгуют: заголовок окна толкучки.
    pub market: String,
    /// Категория без множителя продаётся по кривой товара.
    #[serde(default)]
    pub multipliers: BTreeMap<CategoryId, f64>,
    /// До этой даты город закрыт (Польша — с 1990–91, Китай — с 1992).
    pub opens: Option<GameDate>,
    /// До этой даты в город попадают только по блату (ГДР и Венгрия до 1990).
    pub connections_only_until: Option<GameDate>,
}

impl City {
    /// Во сколько раз цена категории в городе отличается от кривой товара.
    pub fn multiplier(&self, category: &CategoryId) -> f64 {
        self.multipliers
            .get(category)
            .copied()
            .unwrap_or(NEUTRAL_MULTIPLIER)
    }

    /// Город открыт в этот день: у него нет даты открытия или она уже наступила.
    pub fn is_open_on(&self, date: GameDate) -> bool {
        self.opens.is_none_or(|opens| opens <= date)
    }
}
