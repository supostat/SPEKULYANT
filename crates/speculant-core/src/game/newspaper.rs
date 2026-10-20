//! Газета недели (§13.3): раз в неделю, в день записи истории цен, — строка в ленте о самом
//! большом сдвиге цены за неделю на толкучках открытых городов, с настоящими ценниками обеих недель.
//! Если на товаре в том городе дефицит или завал, газета называет и его.

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
            .map(|shock| if shock.factor > 1.0 { DEFICIT } else { GLUT });
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
                let ratio = f64::from(now.real_kopecks?) / f64::from(before.real_kopecks?);
                Some(Shift {
                    city,
                    good,
                    before,
                    now,
                    size: ratio.ln().abs(),
                })
            })
            .max_by(|left, right| left.size.total_cmp(&right.size))
    }
}
