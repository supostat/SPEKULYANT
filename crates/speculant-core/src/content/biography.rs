use serde::Deserialize;

use super::CityId;

/// Стартовая биография (§11.1).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Biography {
    pub title: String,
    pub description: String,
    pub start_city: CityId,
    pub starts_unlocked: bool,
}
