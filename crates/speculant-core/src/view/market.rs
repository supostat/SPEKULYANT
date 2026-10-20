use serde::{Deserialize, Serialize};

use super::{CityPriceView, PriceChartView};

/// Толкучка текущего города: место и строки товаров с ценами следующей штуки в номинале, итогами
/// сделки на любое количество, запасом толкучки, тем, сколько товара при игроке, графиком цены за
/// последний год и ценами по городам.
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
    /// Сколько стоит покупка 1, 2, … штук подряд — до всего запаса толкучки: каждая следующая
    /// штука дороже.
    pub buy_totals: Vec<String>,
    /// Сколько выручит продажа 1, 2, … штук подряд — до всего, что есть у игрока: каждая следующая
    /// штука дешевле.
    pub sell_totals: Vec<String>,
    /// Цена покупки по неделям за последний год — для графика.
    pub chart: PriceChartView,
    /// Цены по городам, открытым сегодня: здесь — первой строкой, остальные — по названию.
    pub cities: Vec<CityPriceView>,
}
