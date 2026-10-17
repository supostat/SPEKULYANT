use serde::{Deserialize, Serialize};

/// Уровень по шкале — индикатор угрозы, черта сотрудника, авторитет: `level` делений из `scale`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LevelView {
    pub label: String,
    pub level: u8,
    pub scale: u8,
}
