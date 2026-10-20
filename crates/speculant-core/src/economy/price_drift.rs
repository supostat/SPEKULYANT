use serde::{Deserialize, Serialize};

use crate::content::{Category, GoodId};
use crate::rng::{fnv1a, fraction_of, mix};
use crate::time::GameDate;

const MONTHS_IN_YEAR: i64 = 12;
/// Опора месяца — его середина: в середине месяца цена ровно такая, какую месяц задаёт.
const MID_MONTH: f64 = 0.5;

/// Дрейф цен (§5.2 п. 6) из `data/economy/drift.toml`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DriftRules {
    /// Насколько месяц может увести цену товара от кривой — доля в обе стороны.
    pub amplitude: f64,
}

/// Дрейф цен мира (§5.2 п. 6): у каждого товара в каждом месяце своё отклонение от кривой —
/// функция сида мира, товара и месяца, — а у категории свой сезон. От середины месяца к середине
/// следующего цена идёт по прямой, поэтому соседние месяцы сшиты без скачка.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceDrift {
    seed: u64,
}

impl PriceDrift {
    pub fn new(seed: u64) -> Self {
        Self { seed }
    }

    /// Во сколько раз дрейф товара и сезон его категории сдвигают цену в этот день.
    pub fn factor(
        &self,
        rules: &DriftRules,
        good: &GoodId,
        category: &Category,
        date: GameDate,
    ) -> f64 {
        let position = date.months_from_start() - MID_MONTH;
        let anchor = position.floor();
        let share = position - anchor;
        let previous = anchor as i64;
        let next = previous + 1;
        let good_key = self.good_key(good);
        let drift = blend(
            month_drift(rules, good_key, previous),
            month_drift(rules, good_key, next),
            share,
        );
        let season = category.season.map_or(1.0, |season| {
            blend(
                season[month_of_year(previous)],
                season[month_of_year(next)],
                share,
            )
        });
        drift * season
    }

    /// Ключ товара в этом мире: из него выводятся отклонения всех месяцев товара.
    fn good_key(&self, good: &GoodId) -> u64 {
        fnv1a(
            self.seed
                .to_le_bytes()
                .into_iter()
                .chain(good.as_str().bytes()),
        )
    }
}

/// Отклонение месяца товара, считая от января 1984 года: равномерно от `1 − amplitude` до
/// `1 + amplitude`.
fn month_drift(rules: &DriftRules, good_key: u64, month: i64) -> f64 {
    let fraction = fraction_of(mix(good_key ^ month.cast_unsigned()));
    1.0 + rules.amplitude * (2.0 * fraction - 1.0)
}

/// Значение на прямой между двумя опорами: `share` — пройденная доля пути от первой ко второй.
fn blend(from: f64, to: f64, share: f64) -> f64 {
    from + (to - from) * share
}

/// Месяц года, январь — 0, для месяца, отсчитанного от января 1984 года.
fn month_of_year(month: i64) -> usize {
    usize::try_from(month.rem_euclid(MONTHS_IN_YEAR)).expect("остаток от деления на 12 — 0–11")
}
