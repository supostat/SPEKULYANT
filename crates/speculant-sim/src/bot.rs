//! Бот — манера торговать, выбирать в событиях, копить деньги, заводить дела и сидеть. Бот
//! играет только командами фасада `Game` и только теми, что правила разрешают: он сам проверяет
//! ценники, наличные и сроки, поэтому отказ ядра — ошибка бота, а не ход игры.

mod arbitrage;
mod choice_style;
mod curiosity;
mod deals;
mod dice;
mod enterprise;
mod hoarding;
mod random_moves;
mod saving;
mod zone_conduct;

use rand_chacha::ChaCha8Rng;
use speculant_core::Game;
use speculant_core::view::EventView;

pub(crate) use arbitrage::Arbitrage;
pub(crate) use choice_style::ChoiceStyle;
pub(crate) use curiosity::Curiosity;
pub(crate) use hoarding::Hoarding;
pub(crate) use random_moves::RandomMoves;
pub(crate) use saving::{PlacePreference, Saving};
pub(crate) use zone_conduct::ZoneConduct;

/// Как бот торгует на толкучке.
pub(crate) enum Trade {
    Random(RandomMoves),
    Arbitrage(Arbitrage),
    Hoard(Hoarding),
}

pub(crate) struct Bot {
    pub trade: Trade,
    pub choices: ChoiceStyle,
    pub saving: Option<Saving>,
    /// Открывает дела, когда наличных хватает, и нанимает управляющих.
    pub enterprise: bool,
    pub zone: ZoneConduct,
}

impl Bot {
    /// Первый день месяца, который бот встретил на воле в городе: вклады, дела и торговля
    /// надолго.
    pub fn live_month(&self, game: &mut Game) {
        if let Some(saving) = &self.saving {
            saving.rebalance(game);
        }
        if self.enterprise {
            enterprise::expand(game);
        }
        if let Trade::Hoard(hoarding) = &self.trade {
            hoarding.trade(game);
        }
    }

    /// День на воле в городе: торговля на каждый день.
    pub fn live_day(&mut self, game: &mut Game, dice: &mut ChaCha8Rng) {
        match &mut self.trade {
            Trade::Random(moves) => moves.make(game, dice),
            Trade::Arbitrage(arbitrage) => arbitrage.trade(game),
            Trade::Hoard(_) => {}
        }
    }

    /// Номер выбора (от 1) в событии, которое ждёт ответа.
    pub fn choose(&self, game: &Game, event: &EventView, dice: &mut ChaCha8Rng) -> usize {
        self.choices.choose(game, event, dice)
    }

    /// Неделя в зоне.
    pub fn serve_week(&self, game: &mut Game) {
        self.zone.serve_week(game);
    }
}
