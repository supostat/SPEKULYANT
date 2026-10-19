use serde::Deserialize;

use crate::economy::Real;

/// Параметры стратегии из `data/rivals.toml`; всё, чего нет, — ноль.
#[derive(Clone, Copy, Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StrategyRules {
    /// Вероятность, что торгующий соперник сегодня торгует.
    #[serde(default)]
    pub trade_chance: f64,
    /// Сколько штук за сделку.
    #[serde(default)]
    pub lot: u32,
    /// Доход в день — дань, взятки — в рублях 1984 года.
    #[serde(default)]
    pub daily_income: Real,
    /// Годовой процент на наличные, начисляется первого числа.
    #[serde(default)]
    pub annual_percent: f64,
    /// Вероятности судьбы в день.
    #[serde(default)]
    pub prison_chance: f64,
    #[serde(default)]
    pub death_chance: f64,
    #[serde(default)]
    pub deputy_chance: f64,
    /// Вероятность в месяц открыть точку своего дела, если хватает денег.
    #[serde(default)]
    pub business_chance: f64,
}
