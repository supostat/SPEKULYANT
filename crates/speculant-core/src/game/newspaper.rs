//! Газета недели (§13.3): раз в неделю, в день записи истории цен, — строка в ленте о самом
//! большом сдвиге цены за неделю на толкучках открытых городов, с настоящими ценниками обеих недель.
//! Если сдвиг объясняет дефицит или завал на товаре в том городе — рост дефицит, падение завал, —
//! газета называет и его.

use std::cmp::Ordering;

use super::Game;
use crate::content::{CityId, GoodId};
use crate::economy::PriceWeek;
use crate::time::GameDate;

const NEWSPAPER: &str = "Газета";
const DEFICIT: &str = "дефицит";
const GLUT: &str = "завал";
const WEEK_AGO: &str = "неделю назад";

/// Сдвиг цены покупки товара в городе за неделю: прошлая и эта запись истории.
struct Shift<'a> {
    city: &'a CityId,
    good: &'a GoodId,
    before: PriceWeek,
    now: PriceWeek,
    /// Куда цена ушла: выросла, упала или стоит.
    direction: Ordering,
    /// Насколько цена ушла, в обе стороны одинаково: модуль логарифма отношения.
    size: f64,
}

impl Game {
    /// Строка газеты о самом большом сдвиге цены за неделю; первой недели жизни сравнивать не с
    /// чем — газеты нет.
    pub(super) fn print_newspaper(&mut self, date: GameDate) {
        let Some(shift) = self.largest_shift(date) else {
            return;
        };
        let content = &self.content;
        let rules = &content.money;
        let tag = |week: PriceWeek| {
            let kopecks = week
                .real_kopecks
                .expect("сдвиг считается только у проданного товара");
            rules.format_money(rules.price_tag(f64::from(kopecks), week.date), week.date)
        };
        let swing = self
            .world
            .markets
            .shocks()
            .iter()
            .find(|shock| {
                shock.city.as_ref() == Some(shift.city)
                    && shock.good == *shift.good
                    && !shock.is_over_on(date)
            })
            .and_then(|shock| match (shock.factor > 1.0, shift.direction) {
                (true, Ordering::Greater) => Some(DEFICIT),
                (false, Ordering::Less) => Some(GLUT),
                _ => None,
            });
        let place = match swing {
            Some(swing) => format!("{}, {swing}", content.cities[shift.city].name),
            None => content.cities[shift.city].name.clone(),
        };
        let line = format!(
            "{NEWSPAPER}: {place} — {}: {}, {WEEK_AGO} {}",
            content.goods[shift.good].name,
            tag(shift.now),
            tag(shift.before)
        );
        self.log(line);
    }

    /// Самый большой сдвиг цены покупки за неделю среди товаров, которые продавались обе недели,
    /// в городах, открытых сегодня.
    fn largest_shift(&self, date: GameDate) -> Option<Shift<'_>> {
        let content = &self.content;
        let history = &self.world.price_history;
        content
            .cities
            .iter()
            .filter(|(_, city)| city.is_open_on(date))
            .flat_map(|(city, _)| content.goods.keys().map(move |good| (city, good)))
            .filter_map(|(city, good)| {
                let [before, now] = history.last_two_weeks(city, good)?;
                let (before_kopecks, now_kopecks) = (before.real_kopecks?, now.real_kopecks?);
                let ratio = f64::from(now_kopecks) / f64::from(before_kopecks);
                Some(Shift {
                    city,
                    good,
                    before,
                    now,
                    direction: now_kopecks.cmp(&before_kopecks),
                    size: ratio.ln().abs(),
                })
            })
            .max_by(|left, right| left.size.total_cmp(&right.size))
    }
}
