use serde::Deserialize;

use crate::economy::Real;
use crate::time::GameDate;

/// Жильё (§11, решение 15.4) из `data/housing/<жильё>.toml`: статус персонажа и место, где лежат
/// деньги «под матрасом».
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Housing {
    pub title: String,
    /// Цена в рублях 1984 года; жильё без цены не продаётся — его дают биография или события.
    pub price: Option<Real>,
    /// С этой даты жильё можно купить; без неё — с начала игры.
    pub on_sale_from: Option<GameDate>,
    /// Шанс в месяц лишиться денег под матрасом — кража, пожар.
    pub stash_risk: f64,
}

impl Housing {
    /// Жильё можно купить в этот день: у него есть цена, и продажа уже открыта.
    pub fn is_on_sale(&self, date: GameDate) -> bool {
        self.price.is_some() && self.on_sale_from.is_none_or(|from| from <= date)
    }
}
