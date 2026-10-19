use speculant_core::content::BiographyId;

use crate::Strategy;

/// Какую жизнь проживёт бот: номер в прогоне, стратегия, биография, сид мира и доля решений,
/// в которых бот любопытен.
#[derive(Clone, Debug, PartialEq)]
pub struct LifePlan {
    pub index: usize,
    pub strategy: Strategy,
    pub biography: BiographyId,
    pub seed: u64,
    pub curiosity: f64,
}
