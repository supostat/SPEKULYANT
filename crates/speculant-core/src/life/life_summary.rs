use super::DealRecord;
use crate::content::{BiographyId, EndingId};
use crate::economy::Real;
use crate::time::GameDate;

/// Итоги жизни (§11.2), снятые в день финала: заметка в духе эпохи — откуда она, заголовок и
/// текст, — капитал в рублях 1984 года, лучшая и худшая сделки, дела на тот день и биографии,
/// которые жизнь открыла.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LifeSummary {
    pub ending: EndingId,
    pub ended: GameDate,
    /// ««ВЕЧЕРНЯЯ МОСКВА» · 5 ноября 1997 · с. 14».
    pub source: String,
    pub headline: String,
    pub text: String,
    pub capital: Real,
    pub best_deal: Option<DealRecord>,
    pub worst_deal: Option<DealRecord>,
    pub businesses: Vec<String>,
    pub discoveries: Vec<BiographyId>,
}
