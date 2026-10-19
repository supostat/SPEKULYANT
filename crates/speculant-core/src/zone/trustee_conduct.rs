use rand_chacha::ChaCha8Rng;

use super::TrusteeRules;
use crate::business::BusinessRules;
use crate::events::EventEngine;

/// Как доверенное лицо ведёт дело месяц, пока хозяин сидит (§12): здесь проверяется, кому игрок
/// доверял.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TrusteeConduct {
    Honest,
    /// Уносит долю выручки месяца по правилам зоны.
    Steals,
    /// Исчезает со всей выручкой месяца; дело остаётся без управляющего.
    Vanishes,
}

impl TrusteeConduct {
    /// Сначала лояльность решает, не исчезнет ли он, затем честность — не ворует ли.
    pub fn draw(rules: &TrusteeRules, loyalty: u8, honesty: u8, rng: &mut ChaCha8Rng) -> Self {
        if EventEngine::roll(rng, BusinessRules::by_trait(&rules.vanish_chances, loyalty)) {
            Self::Vanishes
        } else if EventEngine::roll(rng, BusinessRules::by_trait(&rules.steal_chances, honesty)) {
            Self::Steals
        } else {
            Self::Honest
        }
    }
}
