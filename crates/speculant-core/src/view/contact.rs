use serde::{Deserialize, Serialize};

/// Человек из связей или соперник: отношение, долг, можно ли до него достучаться (у соперника —
/// его судьба) и история знакомства.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactView {
    pub contact: String,
    pub name: String,
    /// Кто он, а у знакомого со своим городом — и город: «боцман сухогруза, Рига».
    pub role: String,
    pub rival: bool,
    pub relation: String,
    pub debt: Option<String>,
    pub availability: String,
    pub history: Vec<String>,
}
