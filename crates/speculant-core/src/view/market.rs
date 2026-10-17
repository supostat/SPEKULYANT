use serde::{Deserialize, Serialize};

/// Толкучка текущего города: место и строки товаров с ценами в номинале и запасом.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketView {
    pub place: String,
    pub goods: Vec<MarketRowView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MarketRowView {
    pub good: String,
    pub name: String,
    pub buy: String,
    pub sell: String,
    pub stock: String,
}
