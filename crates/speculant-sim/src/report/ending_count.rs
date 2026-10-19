use serde::Serialize;
use speculant_core::content::EndingId;

/// Сколько жизней кончилось этим финалом.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct EndingCount {
    pub ending: EndingId,
    pub title: String,
    pub lives: usize,
}
