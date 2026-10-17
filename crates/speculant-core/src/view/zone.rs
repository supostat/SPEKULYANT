use serde::{Deserialize, Serialize};

use super::{LevelView, PocketItemView};

/// Зона: срок, авторитет, своя экономика и доверенные лица на воле.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneView {
    pub camp: String,
    pub sentence: String,
    pub authority: LevelView,
    pub stash: Vec<PocketItemView>,
    pub trustees: Vec<String>,
}
