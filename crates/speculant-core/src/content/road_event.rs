use serde::Deserialize;

use super::EventId;

/// Дорожное событие маршрута (§4) и его вес среди дорожных событий этого маршрута.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RoadEvent {
    pub event: EventId,
    pub weight: u32,
}
