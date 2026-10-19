//! Кубик бота — свой поток случая, отдельный от потоков мира.

use rand_chacha::ChaCha8Rng;
use speculant_core::events::EventEngine;

/// Номер от нуля до `count`, не включая его.
pub(crate) fn index(dice: &mut ChaCha8Rng, count: usize) -> usize {
    assert!(count > 0, "кубик бросают, когда есть из чего выбрать");
    ((EventEngine::fraction(dice) * count as f64) as usize).min(count - 1)
}

pub(crate) fn chance(dice: &mut ChaCha8Rng, probability: f64) -> bool {
    EventEngine::roll(dice, probability)
}
