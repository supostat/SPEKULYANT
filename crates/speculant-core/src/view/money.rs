use serde::{Deserialize, Serialize};

/// Деньги по местам хранения и курсы валют.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoneyView {
    pub places: Vec<MoneyPlaceView>,
    pub rates: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoneyPlaceView {
    pub place: String,
    pub title: String,
    pub amount: String,
    pub risk: String,
}
