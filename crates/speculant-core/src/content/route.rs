use serde::Deserialize;

use super::CityId;
use crate::economy::Real;

/// Направление поездки из `data/routes.toml`: дни в пути и стоимость в рублях 1984 года.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Route {
    pub from: CityId,
    pub to: CityId,
    pub days: u32,
    pub fare: Real,
}
