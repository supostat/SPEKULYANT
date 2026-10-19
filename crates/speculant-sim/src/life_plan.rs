use speculant_core::content::BiographyId;

use crate::Strategy;

/// Какую жизнь проживёт бот: номер в прогоне, стратегия, биография и сид мира.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifePlan {
    pub index: usize,
    pub strategy: Strategy,
    pub biography: BiographyId,
    pub seed: u64,
}
