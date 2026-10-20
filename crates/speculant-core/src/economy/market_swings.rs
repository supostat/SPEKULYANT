use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::Rng;
use serde::Deserialize;

use super::Markets;
use crate::content::{CityId, Content, GoodId};
use crate::events::EventEngine;
use crate::history::DayRange;
use crate::time::GameDate;

/// Дефициты и завалы толкучек (§5.2 п. 4) из `data/economy/swings.toml`: рынок сам по сиду мира
/// рождает временные шоки «город × товар»; о них говорит лента города и слухи других городов.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SwingRules {
    /// Вероятность, что в этот день где-то случится дефицит или завал.
    pub daily_chance: f64,
    /// Доля дефицитов среди них; остальное — завалы.
    pub deficit_share: f64,
    /// Во сколько раз дефицит поднимает цену.
    pub deficit_factor: FactorRange,
    /// Во сколько раз завал опускает цену — множители меньше единицы.
    pub glut_factor: FactorRange,
    /// Сколько дней держится дефицит или завал.
    pub days: DayRange,
    /// Доля слухов, которые называют город верно; остальные путают город.
    pub rumor_accuracy: f64,
    /// Сколько самых свежих слухов о ценах помнит окно «Слухи».
    pub rumors: usize,
}

/// Разброс множителя цены: от `from` до `to`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FactorRange {
    pub from: f64,
    pub to: f64,
}

/// Дефицит или завал, родившийся сегодня: город, товар, множитель цены и срок.
#[derive(Clone, Debug, PartialEq)]
pub struct Swing {
    pub city: CityId,
    pub good: GoodId,
    pub factor: f64,
    pub days: u32,
}

impl Swing {
    /// Дефицит или завал дня: с вероятностью правил — в открытом городе с товаром, который уже
    /// продаётся, всё по сиду; `None` — день прошёл спокойно, продавать ещё нечего или у товара в
    /// этом городе уже свой шок: два дефицита не множатся.
    pub fn roll(
        content: &Content,
        markets: &Markets,
        rng: &mut ChaCha8Rng,
        date: GameDate,
    ) -> Option<Self> {
        let rules = &content.swings;
        if !EventEngine::roll(rng, rules.daily_chance) {
            return None;
        }
        let cities: Vec<&CityId> = content
            .cities
            .iter()
            .filter(|(_, city)| city.is_open_on(date))
            .map(|(id, _)| id)
            .collect();
        let goods: Vec<&GoodId> = content
            .goods
            .keys()
            .filter(|good| Markets::is_on_sale(content, good, date))
            .collect();
        if goods.is_empty() {
            return None;
        }
        let city = cities[index(rng, cities.len())].clone();
        let good = goods[index(rng, goods.len())].clone();
        let range = if EventEngine::roll(rng, rules.deficit_share) {
            rules.deficit_factor
        } else {
            rules.glut_factor
        };
        let factor = range.from + (range.to - range.from) * EventEngine::fraction(rng);
        let spread = u64::from(rules.days.to - rules.days.from + 1);
        let days = rules.days.from + u32::try_from(rng.next_u64() % spread).expect("срок мал");
        let shocked = markets.shocks().iter().any(|shock| {
            shock.city.as_ref() == Some(&city) && shock.good == good && !shock.is_over_on(date)
        });
        if shocked {
            return None;
        }
        Some(Self {
            city,
            good,
            factor,
            days,
        })
    }

    /// Дефицит поднимает цену, завал опускает.
    pub fn is_deficit(&self) -> bool {
        self.factor > 1.0
    }
}

/// Равновероятный номер от нуля до `count`, не включая его.
fn index(rng: &mut ChaCha8Rng, count: usize) -> usize {
    usize::try_from(rng.next_u64() % u64::try_from(count).expect("городов и товаров мало"))
        .expect("номер меньше числа")
}
