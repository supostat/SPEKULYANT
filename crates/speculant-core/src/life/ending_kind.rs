use serde::{Deserialize, Serialize};

/// Каким бывает конец жизни (§11.2).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndingKind {
    /// Бандиты, несчастный случай, здоровье, пьянство.
    Death,
    /// Срок, который не пересидеть.
    Prison,
    /// Разорение с долгами и бегство.
    Ruin,
    /// Эмиграция или покой — по своей воле.
    Departure,
}
