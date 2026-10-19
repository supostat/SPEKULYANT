use serde::{Deserialize, Serialize};

use super::PriceNote;

/// Что известно о цене товара: дешевле всего купленная штука и дороже всего проданная.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GoodPrices {
    pub cheapest_buy: Option<PriceNote>,
    pub dearest_sell: Option<PriceNote>,
}

impl GoodPrices {
    pub fn bought(&mut self, note: PriceNote) {
        if self
            .cheapest_buy
            .as_ref()
            .is_none_or(|cheapest| note.price < cheapest.price)
        {
            self.cheapest_buy = Some(note);
        }
    }

    pub fn sold(&mut self, note: PriceNote) {
        if self
            .dearest_sell
            .as_ref()
            .is_none_or(|dearest| note.price > dearest.price)
        {
            self.dearest_sell = Some(note);
        }
    }

    /// Знание другой жизни: остаётся лучшее из двух.
    pub fn merge(&mut self, other: &Self) {
        if let Some(note) = &other.cheapest_buy {
            self.bought(note.clone());
        }
        if let Some(note) = &other.dearest_sell {
            self.sold(note.clone());
        }
    }
}
