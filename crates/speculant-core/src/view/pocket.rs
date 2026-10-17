use serde::{Deserialize, Serialize};

use super::LevelView;

/// Окно «Карман»: что при себе и главная угроза эпохи (ОБХСС, рэкет, налоговая).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PocketView {
    pub items: Vec<PocketItemView>,
    pub danger: Option<LevelView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PocketItemView {
    pub label: String,
    pub amount: String,
}
