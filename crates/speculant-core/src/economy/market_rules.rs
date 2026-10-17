use serde::Deserialize;

/// Правила толкучки (§5.2 п. 3) из `data/economy/market.toml`: как цена отвечает на запас и как
/// рынок приходит в себя после крупной сделки.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MarketRules {
    /// Разница цен покупки и продажи — доля средней цены.
    pub spread: f64,
    /// Доля отклонения запаса от обычного, которую рынок выбирает за день.
    pub recovery: f64,
    /// Случайное колебание запаса за день — доля обычного запаса.
    pub daily_noise: f64,
}
