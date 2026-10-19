use crate::content::{BusinessKind, CityId, Content};
use crate::economy::Markets;
use crate::time::GameDate;

const PERCENT: f64 = 100.0;

/// Как работает дело: сколько точек, сдвиг выручки по прогнозу, хватка управляющего, уровень
/// зарплаты и сколько точек того же типа держат в городе соперники.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Operation {
    pub points: u32,
    pub revenue_percent: i32,
    pub skill: u8,
    pub wage_percent: u32,
    pub competitors: u32,
}

/// Деньги дела за полный месяц работы в копейках 1984 года: выручка, аренда и зарплата. Доли
/// крыше и налог берутся с выручки при расчёте месяца.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MonthlyBudget {
    pub income: f64,
    pub rent: f64,
    pub wages: f64,
}

impl MonthlyBudget {
    /// Выручка точки — продажи товара по средней цене города, умноженные на долю, которая остаётся
    /// делу, плюс оборот без товара; она растёт с хваткой выше середины шкалы, сдвигается политиками
    /// и делится с конкурентами. Товар, которого ещё нет в продаже, ничего не приносит.
    pub fn plan(
        content: &Content,
        markets: &Markets,
        kind: &BusinessKind,
        city: &CityId,
        operation: Operation,
        date: GameDate,
    ) -> Self {
        let rules = &content.business;
        let sales: f64 = kind
            .goods
            .iter()
            .filter_map(|(good, quantity)| {
                markets
                    .normal_kopecks(content, city, good, date)
                    .map(|price| price * f64::from(*quantity))
            })
            .sum();
        let per_point = sales * kind.margin + kind.income.kopecks() as f64;
        let skill_steps = f64::from(operation.skill) - f64::from(rules.neutral_trait);
        let skill_factor = (1.0 + skill_steps * rules.skill_step).max(0.0);
        let policy_factor = (1.0 + f64::from(operation.revenue_percent) / PERCENT).max(0.0);
        let competition_factor = 1.0 / (1.0 + rules.competition * f64::from(operation.competitors));
        let points = f64::from(operation.points);
        Self {
            income: per_point * points * skill_factor * policy_factor * competition_factor,
            rent: kind.rent.kopecks() as f64 * points,
            wages: kind.wages.kopecks() as f64 * points * f64::from(operation.wage_percent)
                / PERCENT,
        }
    }
}
