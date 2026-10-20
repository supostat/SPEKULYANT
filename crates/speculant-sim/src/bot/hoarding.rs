use speculant_core::Game;
use speculant_core::content::GoodId;
use speculant_core::economy::{Currency, Markets, Money};

use super::deals;

/// Знаток истории цен — игрок, который помнит прошлые жизни: раз в месяц продаёт товар, кривая
/// которого пошла вниз, и вкладывает долю наличных в товар, который подорожает сильнее всех.
pub(crate) struct Hoarding {
    /// Доля наличных, которую бот вкладывает в товар за раз.
    pub stake: f64,
    /// На сколько дней вперёд бот смотрит в кривую цены.
    pub outlook_days: u32,
    /// Во сколько раз товар должен подорожать за это время, чтобы его стоило копить.
    pub least_growth: f64,
}

impl Hoarding {
    pub fn trade(&self, game: &mut Game) {
        self.sell_falling(game);
        if let Some((good, quantity)) = self.best_buy(game) {
            deals::buy(game, &good, quantity);
        }
    }

    /// Во сколько раз кривая товара вырастет за горизонт; `None`, пока товара нет.
    fn growth(&self, game: &Game, good: &GoodId) -> Option<f64> {
        let curve = &game.content().goods[good].curve;
        let today = game.today();
        let now = curve.kopecks_on(today)?;
        let later = curve.kopecks_on(today.after_days(self.outlook_days))?;
        Some(later / now)
    }

    fn sell_falling(&self, game: &mut Game) {
        let falling: Vec<(GoodId, u32)> = game
            .player()
            .pocket()
            .goods()
            .filter(|(good, _)| Markets::is_on_sale(game.content(), good, game.today()))
            .filter(|(good, _)| self.growth(game, good).is_some_and(|growth| growth < 1.0))
            .map(|(good, quantity)| (good.clone(), quantity))
            .collect();
        for (good, quantity) in falling {
            deals::sell(game, &good, quantity);
        }
    }

    /// Товар, который вырастет сильнее всех, не меньше порога бота и хотя бы штукой укладывается в
    /// ставку, — и сколько его взять.
    fn best_buy(&self, game: &Game) -> Option<(GoodId, u32)> {
        let content = game.content();
        let today = game.today();
        let city = game.city();
        let markets = game.markets();
        let cash = game.player().money().cash();
        let budget = Money::new(
            Currency::Ruble,
            (cash.minor_units() as f64 * self.stake) as i64,
        );
        let mut growing: Vec<(&GoodId, f64)> = content
            .goods
            .keys()
            .filter(|good| markets.city(city).stock(good) > 0)
            .filter_map(|good| Some((good, self.growth(game, good)?)))
            .filter(|(_, growth)| *growth >= self.least_growth)
            .collect();
        growing.sort_by(|left, right| right.1.total_cmp(&left.1));
        growing.into_iter().find_map(|(good, _)| {
            let stock = markets.city(city).stock(good);
            let tags = markets.buy_tags(content, city, good, stock, today)?;
            let quantity = deals::units_within(&tags, budget);
            (quantity > 0).then(|| (good.clone(), quantity))
        })
    }
}

#[cfg(test)]
mod tests {
    use speculant_core::content::GoodId;

    use super::Hoarding;
    use crate::test_worlds::{date, live_until, new_life, real_data};

    const HOARDER: Hoarding = Hoarding {
        stake: 0.8,
        outlook_days: 730,
        least_growth: 1.3,
    };

    fn vodka() -> GoodId {
        GoodId::new("vodka")
    }

    fn held(game: &speculant_core::Game) -> Vec<&GoodId> {
        game.player()
            .pocket()
            .goods()
            .map(|(good, _)| good)
            .collect()
    }

    #[test]
    fn knowing_the_dry_law_the_hoarder_buys_vodka_in_1984_and_sells_it_when_the_curve_turns() {
        let mut game = new_life(&real_data(), "sailor");
        HOARDER.trade(&mut game);
        assert_eq!(
            held(&game),
            vec![&vodka()],
            "за два года водка дорожает сильнее всего — к 1986-му в 1,7 раза"
        );
        let bought = game.player().pocket().quantity(&vodka());

        live_until(&mut game, date(1986, 6, 1));
        HOARDER.trade(&mut game);
        assert!(
            game.player().pocket().quantity(&vodka()) >= bought,
            "через два года водка ещё дороже"
        );

        live_until(&mut game, date(1987, 6, 1));
        HOARDER.trade(&mut game);
        assert_eq!(
            game.player().pocket().quantity(&vodka()),
            0,
            "через два года водка дешевле, чем сейчас, — продана за полгода до пика 1988-го"
        );
    }

    #[test]
    fn a_hoarder_short_of_cash_for_vodka_takes_the_next_best_growth() {
        let half_stake = Hoarding {
            stake: 0.5,
            ..HOARDER
        };
        let mut game = new_life(&real_data(), "student");
        half_stake.trade(&mut game);
        assert_eq!(
            held(&game),
            vec![&GoodId::new("cognac")],
            "ящик водки дороже половины денег студента, коньяк к 1986-му дорожает в полтора раза"
        );
    }
}
