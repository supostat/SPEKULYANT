use serde::{Deserialize, Serialize};

/// Цены товара в городе, какими их знает игрок: здесь — нынешние, там, откуда он уезжал, — с
/// ценников дня отъезда в номинале того дня; пусто — товара тогда не продавали или игрок там не
/// бывал. Самая низкая цена покупки и самая высокая цена продажи отмечены, когда их есть с чем
/// сравнить.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityPriceView {
    pub name: String,
    pub here: bool,
    /// День, когда игрок видел эти цены: здесь — сегодня; `None` — он там не бывал.
    pub seen: Option<String>,
    pub buy: String,
    pub sell: String,
    pub cheapest: bool,
    pub dearest: bool,
}
