use serde::Deserialize;

use super::CityId;
use crate::economy::Real;

/// Стартовая биография (§11.1): где и с чем начинается жизнь.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Biography {
    pub title: String,
    pub description: String,
    pub start_city: CityId,
    /// Наличные на старте в рублях 1984 года.
    pub start_cash: Real,
    pub starts_unlocked: bool,
}
