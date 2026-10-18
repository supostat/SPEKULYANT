use serde::{Deserialize, Serialize};

/// Человек из связей или соперник: отношение, долг, можно ли до него достучаться и история знакомства.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContactView {
    pub contact: String,
    pub name: String,
    pub role: String,
    pub relation: String,
    pub debt: Option<String>,
    pub availability: String,
    pub history: Vec<String>,
}
