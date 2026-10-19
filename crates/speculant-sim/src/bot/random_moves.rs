use rand_chacha::ChaCha8Rng;
use speculant_core::content::{GoodId, Route};
use speculant_core::economy::{Currency, Markets};
use speculant_core::travel::Access;
use speculant_core::{Command, Game};

use super::{deals, dice};

/// Игрок наугад — точка отсчёта баланса: в день с этим шансом покупает случайное, продаёт
/// случайное или едет куда-нибудь.
pub(crate) struct RandomMoves {
    pub act_chance: f64,
}

impl RandomMoves {
    pub fn make(&self, game: &mut Game, dice: &mut ChaCha8Rng) {
        if !dice::chance(dice, self.act_chance) {
            return;
        }
        match dice::index(dice, 3) {
            0 => buy_something(game, dice),
            1 => sell_something(game, dice),
            _ => travel_somewhere(game, dice),
        }
    }
}

/// Случайный товар толкучки — сколько-нибудь, на что хватает наличных.
fn buy_something(game: &mut Game, dice: &mut ChaCha8Rng) {
    let content = game.content();
    let today = game.today();
    let markets = game.markets();
    let on_shelf: Vec<&GoodId> = content
        .goods
        .keys()
        .filter(|good| Markets::is_on_sale(content, good, today))
        .filter(|good| markets.city(game.city()).stock(good) > 0)
        .collect();
    if on_shelf.is_empty() {
        return;
    }
    let good = on_shelf[dice::index(dice, on_shelf.len())].clone();
    let stock = markets.city(game.city()).stock(&good);
    let tags = markets
        .buy_tags(content, game.city(), &good, stock, today)
        .expect("товар в продаже — отобран выше");
    let affordable = deals::units_within(&tags, game.player().money().cash());
    if affordable == 0 {
        return;
    }
    let quantity = 1 + dice::index(dice, affordable as usize) as u32;
    deals::buy(game, &good, quantity);
}

/// Случайный товар из кармана — сколько-нибудь.
fn sell_something(game: &mut Game, dice: &mut ChaCha8Rng) {
    let held: Vec<(GoodId, u32)> = game
        .player()
        .pocket()
        .goods()
        .filter(|(good, _)| Markets::is_on_sale(game.content(), good, game.today()))
        .map(|(good, quantity)| (good.clone(), quantity))
        .collect();
    if held.is_empty() {
        return;
    }
    let (good, held) = &held[dice::index(dice, held.len())];
    let quantity = 1 + dice::index(dice, *held as usize) as u32;
    deals::sell(game, good, quantity);
}

/// Случайный открытый маршрут из города, проезд по которому по карману.
fn travel_somewhere(game: &mut Game, dice: &mut ChaCha8Rng) {
    let content = game.content();
    let today = game.today();
    let cash = game.player().money().cash();
    let affordable: Vec<&Route> = content
        .routes
        .iter()
        .filter(|route| &route.from == game.city())
        .filter(|route| Access::of(route, content, today) == Access::Open)
        .filter(|route| cash.covers(content.money.from_real(route.fare, Currency::Ruble, today)))
        .collect();
    if affordable.is_empty() {
        return;
    }
    let destination = affordable[dice::index(dice, affordable.len())]
        .to
        .to_string();
    game.apply(Command::Travel { destination })
        .expect("маршрут открыт, а проезд по карману");
}

#[cfg(test)]
mod tests {
    use speculant_core::rng::WorldRng;

    use super::RandomMoves;
    use crate::test_worlds::{new_life, real_data};

    #[test]
    fn a_random_player_buys_sells_and_travels_only_as_the_rules_allow() {
        let mut game = new_life(&real_data(), "student");
        let mut world = WorldRng::new(3);
        let dice = world.stream("bot");
        let moves = RandomMoves { act_chance: 1.0 };
        let day = game.content().time.clock.day_milliseconds_at(1);
        let (mut bought, mut sold, mut traveled) = (false, false, false);
        for _ in 0..120 {
            if game.events().pending().is_none() && game.journey().is_none() {
                let held: u32 = game.player().pocket().goods().map(|(_, count)| count).sum();
                moves.make(&mut game, dice);
                let now: u32 = game.player().pocket().goods().map(|(_, count)| count).sum();
                bought |= now > held;
                sold |= now < held;
                traveled |= game.journey().is_some();
            }
            game.advance(day);
        }
        assert!(
            bought && sold && traveled,
            "за 120 дней было всё: {bought} {sold} {traveled}"
        );
    }
}
