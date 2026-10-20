use serde::{Deserialize, Serialize};

/// График цены товара на толкучке города: цена покупки по неделям за последний год от старых к
/// новым — в копейках номинала своей недели, как её показывал ценник; `None` — товара ещё не было.
/// Подписи готовы к показу: самая низкая и самая высокая цена и дни первой и последней недели;
/// без истории — пустые.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PriceChartView {
    pub weeks: Vec<Option<i64>>,
    pub low: String,
    pub high: String,
    pub from: String,
    pub to: String,
}
