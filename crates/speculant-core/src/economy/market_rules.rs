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

impl MarketRules {
    /// Цена покупки к средней цене: половина разницы цен сверху.
    pub fn markup(&self) -> f64 {
        1.0 + self.spread / 2.0
    }

    /// Цена продажи к средней цене: половина разницы цен снизу.
    pub fn markdown(&self) -> f64 {
        1.0 - self.spread / 2.0
    }

    /// Везти выгодно: товар, купленный там, где его цена — `bought` от кривой, и проданный там, где
    /// она — `sold`, при обычном запасе окупает разницу цен покупки и продажи.
    pub fn pays_to_carry(&self, bought: f64, sold: f64) -> bool {
        sold * self.markdown() > bought * self.markup()
    }
}

#[cfg(test)]
mod tests {
    use super::MarketRules;

    const RULES: MarketRules = MarketRules {
        spread: 0.1,
        recovery: 0.25,
        daily_noise: 0.0,
    };

    #[test]
    fn carrying_pays_only_when_the_difference_covers_the_spread() {
        assert!(RULES.pays_to_carry(0.6, 1.0), "там на 40% дешевле");
        assert!(
            !RULES.pays_to_carry(0.95, 1.0),
            "5% разницы съедает разница цен покупки и продажи"
        );
        assert!(!RULES.pays_to_carry(1.0, 0.6), "обратно — в убыток");
    }
}
