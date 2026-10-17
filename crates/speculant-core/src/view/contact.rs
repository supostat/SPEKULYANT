use serde::{Deserialize, Serialize};

/// Человек из связей или соперник: отношение, долг и история знакомства.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactView {
    pub contact: String,
    pub name: String,
    pub role: String,
    pub relation: String,
    pub debt: Option<String>,
    pub history: Vec<String>,
}
