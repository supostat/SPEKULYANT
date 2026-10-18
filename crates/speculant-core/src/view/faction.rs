use serde::{Deserialize, Serialize};

/// Репутация у фракции, которая уже есть в мире: уровень от `-limit` до `limit`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionView {
    pub faction: String,
    pub title: String,
    pub level: i32,
    pub limit: u8,
}
