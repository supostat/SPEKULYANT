use serde::Deserialize;

use super::{Conditions, Consequences, EpochId, EventId, GoodId};
use crate::economy::Real;

/// Событие (§10.1): условия, вес среди подходящих, текст с подстановками `{npc}`, `{price}`, `{sum}`
/// и выборы с последствиями и отложенными продолжениями.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub title: String,
    pub weight: u32,
    #[serde(default)]
    pub once: bool,
    /// Сколько дней повторяемое событие отдыхает после того, как случилось; без поля — по
    /// правилам событий.
    pub cooldown_days: Option<u32>,
    #[serde(default)]
    pub conditions: Conditions,
    /// Сумма события в рублях 1984 года — подстановка `{sum}` в номинале текущей валюты.
    pub sum: Option<Real>,
    /// Товар, чья цена на толкучке города — подстановка `{price}`.
    pub price_of: Option<GoodId>,
    pub text: String,
    pub choices: Vec<Choice>,
    /// Эпоха из пути `data/events/<эпоха>/`; у событий зоны (`data/zone/events/`) её нет.
    #[serde(skip)]
    pub epoch: Option<EpochId>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Choice {
    pub text: String,
    #[serde(default)]
    pub conditions: Conditions,
    #[serde(default)]
    pub consequences: Consequences,
    pub outcome: Option<String>,
    #[serde(default)]
    pub follow_ups: Vec<FollowUp>,
}

/// Отложенный эффект: событие-продолжение через `after_days` дней с вероятностью `chance`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FollowUp {
    pub event: EventId,
    pub after_days: u32,
    pub chance: f64,
}
