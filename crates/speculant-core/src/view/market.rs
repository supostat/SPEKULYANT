use serde::{Deserialize, Serialize};

use super::{CityPriceView, PriceChartView};

/// Толкучка текущего города: место и строки товаров с ценами в номинале, запасом толкучки, тем,
/// сколько товара при игроке, графиком цены за последний год и ценами по городам.
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
    /// Сколько этого товара в кармане игрока; пусто — нисколько.
    pub held: String,
    /// Цена покупки по неделям за последний год — для графика.
    pub chart: PriceChartView,
    /// Цены по городам, открытым сегодня: здесь — первой строкой, остальные — по названию.
    pub cities: Vec<CityPriceView>,
}
