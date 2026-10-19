use std::collections::BTreeMap;

use serde::Deserialize;

use super::{Strategy, StrategyRules};
use crate::economy::Real;

/// Правила соперников из `data/rivals.toml`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RivalRules {
    /// Сколько соперников в мире: от и до, сколько позволит пул шаблонов.
    pub min_rivals: u32,
    pub max_rivals: u32,
    /// Капитал ниже этого, в рублях 1984 года, — разорился.
    pub ruin_below: Real,
    /// Срок соперника в зоне.
    pub prison_days: u32,
    pub strategies: BTreeMap<Strategy, StrategyRules>,
}
