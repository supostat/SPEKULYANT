//! Сделки бота на толкучке: он покупает и продаёт только то, что сам просчитал по ценникам.

use speculant_core::content::GoodId;
use speculant_core::economy::Money;
use speculant_core::{Command, Game};

pub(crate) fn buy(game: &mut Game, good: &GoodId, quantity: u32) {
    game.apply(Command::Buy {
        good: good.to_string(),
        quantity,
    })
    .expect("бот покупает по ценникам, которые просчитал");
}

pub(crate) fn sell(game: &mut Game, good: &GoodId, quantity: u32) {
    game.apply(Command::Sell {
        good: good.to_string(),
        quantity,
    })
    .expect("бот продаёт то, что лежит в кармане");
}

/// Сколько штук подряд по этим ценникам укладывается в бюджет.
pub(crate) fn units_within(tags: &[Money], budget: Money) -> u32 {
    let mut spent = 0;
    let affordable = tags
        .iter()
        .take_while(|tag| {
            spent += tag.minor_units();
            spent <= budget.minor_units()
        })
        .count();
    u32::try_from(affordable).expect("ценников не больше запаса толкучки")
}

#[cfg(test)]
mod tests {
    use speculant_core::economy::{Currency, Money};

    use super::units_within;

    fn rubles(kopecks: i64) -> Money {
        Money::new(Currency::Ruble, kopecks)
    }

    #[test]
    fn the_budget_buys_the_tags_in_a_row_while_their_sum_fits() {
        let tags = [rubles(100), rubles(110), rubles(121)];
        assert_eq!(units_within(&tags, rubles(99)), 0);
        assert_eq!(units_within(&tags, rubles(210)), 2);
        assert_eq!(units_within(&tags, rubles(230)), 2);
        assert_eq!(units_within(&tags, rubles(331)), 3);
    }
}
