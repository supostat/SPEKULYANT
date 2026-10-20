use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::Money;
use crate::content::{CityId, GoodId};
use crate::time::GameDate;

/// Цены толкучек, какими их видел игрок (§4): уезжая из города, он помнит ценники покупки и
/// продажи каждого товара в номинале того дня. Новый отъезд из города заменяет прошлую память о
/// нём; товара, которого тогда ещё не продавали, в памяти нет. Из зоны игрок выходит в тот же
/// город, где его взяли, поэтому арест памяти не нужен.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenPrices {
    cities: BTreeMap<CityId, SeenMarket>,
}

/// Толкучка города в день отъезда.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenMarket {
    pub date: GameDate,
    pub goods: BTreeMap<GoodId, SeenPrice>,
}

/// Ценники следующей штуки товара в день отъезда.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenPrice {
    pub buy: Money,
    pub sell: Money,
}

impl SeenPrices {
    pub fn remember(&mut self, city: CityId, market: SeenMarket) {
        self.cities.insert(city, market);
    }

    /// Толкучка города, какой игрок видел её, уезжая; `None` — он оттуда ещё не уезжал.
    pub fn market(&self, city: &CityId) -> Option<&SeenMarket> {
        self.cities.get(city)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{SeenMarket, SeenPrice, SeenPrices};
    use crate::content::{CityId, GoodId};
    use crate::economy::{Currency, Money};
    use crate::time::GameDate;

    fn market(day: u32, buy_kopecks: i64) -> SeenMarket {
        let price = SeenPrice {
            buy: Money::new(Currency::Ruble, buy_kopecks),
            sell: Money::new(Currency::Ruble, buy_kopecks - 1),
        };
        SeenMarket {
            date: GameDate::START.after_days(day),
            goods: BTreeMap::from([(GoodId::new("jeans"), price)]),
        }
    }

    #[test]
    fn a_later_departure_replaces_the_earlier_memory_of_the_city() {
        let moscow = CityId::new("moscow");
        let mut seen = SeenPrices::default();
        assert_eq!(seen.market(&moscow), None, "игрок ещё не уезжал");
        seen.remember(moscow.clone(), market(3, 19_400));
        seen.remember(moscow.clone(), market(40, 21_000));
        assert_eq!(seen.market(&moscow), Some(&market(40, 21_000)));
    }
}
