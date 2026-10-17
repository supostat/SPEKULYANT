use serde::{Deserialize, Serialize};

/// Стартовая биография на экране выбора: закрытые показываются с условием открытия.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BiographyView {
    pub biography: String,
    pub title: String,
    pub start: String,
    pub unlocked: bool,
}
