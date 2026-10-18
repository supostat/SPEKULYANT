use serde::Deserialize;

/// Правила событий из `data/event_rules.toml`.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventRules {
    /// Вероятность, что в спокойный день что-то случится.
    pub daily_chance: f64,
    /// Вероятность события в каждый день пути.
    pub road_daily_chance: f64,
    /// Сколько дней повторяемое событие отдыхает, если у него нет своего `cooldown_days`.
    pub cooldown_days: u32,
}
