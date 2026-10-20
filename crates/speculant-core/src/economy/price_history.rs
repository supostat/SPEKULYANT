use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::content::{CityId, GoodId};
use crate::time::GameDate;

/// Дней между записями истории: запись — раз в неделю.
pub const WEEK_DAYS: u32 = 7;
/// Сколько последних недель помнит история: год — как раз ширина графика на экране.
pub const HISTORY_WEEKS: usize = 52;

/// История цен толкучек (§5): раз в неделю — цена покупки штуки каждого товара в каждом городе в
/// копейках 1984 года, какой она была на ценнике в тот день; помнится последний год. Товара, которого
/// ещё не было в продаже, в неделе нет.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceHistory {
    /// День последней записи; все города и товары записываются в один день.
    newest: Option<GameDate>,
    weeks: BTreeMap<CityId, BTreeMap<GoodId, Vec<Option<u32>>>>,
}

/// Неделя истории: день записи и цена покупки в копейках 1984 года, если товар уже продавался.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PriceWeek {
    pub date: GameDate,
    pub real_kopecks: Option<u32>,
}

impl PriceHistory {
    /// Записывает неделю: цена каждого товара в каждом городе; неделя старше года уходит. Ключи
    /// копируются только в первую запись города и товара.
    pub fn record<'a>(
        &mut self,
        date: GameDate,
        prices: impl IntoIterator<Item = (&'a CityId, &'a GoodId, Option<u32>)>,
    ) {
        self.newest = Some(date);
        for (city, good, price) in prices {
            if !self.weeks.contains_key(city) {
                self.weeks.insert(city.clone(), BTreeMap::new());
            }
            let goods = self
                .weeks
                .get_mut(city)
                .expect("город заведён строкой выше");
            if !goods.contains_key(good) {
                goods.insert(good.clone(), Vec::with_capacity(HISTORY_WEEKS + 1));
            }
            let weeks = goods.get_mut(good).expect("товар заведён строкой выше");
            weeks.push(price);
            let excess = weeks.len().saturating_sub(HISTORY_WEEKS);
            weeks.drain(..excess);
        }
    }

    /// Недели товара в городе от старых к новым с днями записи.
    pub fn weeks(&self, city: &CityId, good: &GoodId) -> Vec<PriceWeek> {
        let (Some(newest), Some(prices)) = (
            self.newest,
            self.weeks.get(city).and_then(|goods| goods.get(good)),
        ) else {
            return Vec::new();
        };
        let count = prices.len();
        prices
            .iter()
            .enumerate()
            .map(|(index, price)| {
                let weeks_back = i32::try_from(count - 1 - index).expect("недель не больше года");
                let days_back = weeks_back * i32::try_from(WEEK_DAYS).expect("неделя мала");
                PriceWeek {
                    date: GameDate::from_day_index(newest.day_index() - days_back),
                    real_kopecks: *price,
                }
            })
            .collect()
    }

    /// День записи: каждый седьмой день от 1 января 1984 года — первого дня каждой жизни.
    pub fn is_record_day(date: GameDate) -> bool {
        date.day_index() % i32::try_from(WEEK_DAYS).expect("неделя мала") == 0
    }
}

#[cfg(test)]
mod tests {
    use super::{HISTORY_WEEKS, PriceHistory, PriceWeek, WEEK_DAYS};
    use crate::content::{CityId, GoodId};
    use crate::time::GameDate;

    fn record(history: &mut PriceHistory, day: u32, price: Option<u32>) {
        let (city, good) = (CityId::new("moscow"), GoodId::new("jeans"));
        history.record(GameDate::START.after_days(day), [(&city, &good, price)]);
    }

    #[test]
    fn weeks_come_from_old_to_new_with_their_record_days() {
        let mut history = PriceHistory::default();
        record(&mut history, 0, None);
        record(&mut history, WEEK_DAYS, Some(19_400));
        assert_eq!(
            history.weeks(&CityId::new("moscow"), &GoodId::new("jeans")),
            [
                PriceWeek {
                    date: GameDate::START,
                    real_kopecks: None,
                },
                PriceWeek {
                    date: GameDate::START.after_days(WEEK_DAYS),
                    real_kopecks: Some(19_400),
                },
            ]
        );
        assert!(
            history
                .weeks(&CityId::new("leningrad"), &GoodId::new("jeans"))
                .is_empty()
        );
    }

    #[test]
    fn only_the_last_year_is_kept() {
        let mut history = PriceHistory::default();
        let total = u32::try_from(HISTORY_WEEKS).expect("год недель") + 3;
        for week in 0..total {
            record(&mut history, week * WEEK_DAYS, Some(week));
        }
        let weeks = history.weeks(&CityId::new("moscow"), &GoodId::new("jeans"));
        assert_eq!(weeks.len(), HISTORY_WEEKS);
        assert_eq!(
            weeks[0].real_kopecks,
            Some(3),
            "три самые старые недели ушли"
        );
        assert_eq!(weeks[0].date, GameDate::START.after_days(3 * WEEK_DAYS));
    }

    #[test]
    fn records_fall_on_every_seventh_day_from_the_start() {
        assert!(PriceHistory::is_record_day(GameDate::START));
        assert!(!PriceHistory::is_record_day(GameDate::START.after_days(1)));
        assert!(PriceHistory::is_record_day(
            GameDate::START.after_days(WEEK_DAYS)
        ));
    }
}
