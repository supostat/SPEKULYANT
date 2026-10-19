use serde::Serialize;
use speculant_core::content::{BiographyId, EndingId, EventId};
use speculant_core::economy::Real;
use speculant_core::time::GameDate;

use crate::Strategy;

/// Чем кончилась жизнь бота.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct LifeOutcome {
    pub strategy: Strategy,
    pub biography: BiographyId,
    pub seed: u64,
    /// Финал; `None` — бот дожил до бесконечной эпохи.
    pub ending: Option<EndingId>,
    /// День финала или первый день бесконечной эпохи; сам он уже не прожит.
    pub last_day: GameDate,
    /// Капитал в рублях 1984 года: из итогов жизни или на день, когда бот дожил.
    pub capital: Real,
    /// Капитал на первый день каждой следующей эпохи, которую жизнь застала, — по порядку эпох.
    pub epoch_capitals: Vec<Real>,
    /// Какие события случились и когда.
    pub events: Vec<(GameDate, EventId)>,
}
