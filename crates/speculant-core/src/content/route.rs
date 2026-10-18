use serde::Deserialize;

use super::{CityId, RoadEvent};
use crate::economy::Real;

/// Направление поездки из `data/routes.toml` (§4): дни в пути, стоимость в рублях 1984 года и
/// дорожные события маршрута с весами — таможня, поезд, ограбление.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub from: CityId,
    pub to: CityId,
    pub days: u32,
    pub fare: Real,
    #[serde(default)]
    pub road_events: Vec<RoadEvent>,
}
