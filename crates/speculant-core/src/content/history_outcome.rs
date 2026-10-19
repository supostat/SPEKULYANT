use serde::Deserialize;

use super::{EventId, FlagId, PriceShock};

/// Исход узла истории (§3.6): вес среди исходов узла, новость в ленту, шоки цен — их сила растёт
/// с масштабом узла, — флаги для условий событий и событие, которое начинается в день узла.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HistoryOutcome {
    pub weight: u32,
    pub news: String,
    #[serde(default)]
    pub price_shocks: Vec<PriceShock>,
    pub event: Option<EventId>,
    #[serde(default)]
    pub set_flags: Vec<FlagId>,
}
