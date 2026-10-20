//! Толкучки в жизни (§5.2 п. 4, §8): дефицит или завал дня и сделки челноков — новости ленты,
//! когда игрок стоит на толкучке этого города.

use super::Game;
use crate::content::CityId;
use crate::economy::Swing;
use crate::npc::{DealSide, RivalDeal};
use crate::time::GameDate;

const SWINGS_STREAM: &str = "swings";
const DEFICIT: &str = "На толкучке дефицит";
const GLUT: &str = "На толкучке завал";
const BOUGHT_UP: &str = "Скупка";
const BROUGHT: &str = "Привоз";

impl Game {
    /// Дефицит или завал дня ложится шоком на толкучку своего города; в городе игрока — новость.
    pub(super) fn roll_market_swing(&mut self, date: GameDate) {
        let stream = self.world.rng.stream(SWINGS_STREAM);
        let Some(swing) = Swing::roll(&self.content, &self.world.markets, stream, date) else {
            return;
        };
        if self.is_at_market_of(&swing.city) {
            let label = if swing.is_deficit() { DEFICIT } else { GLUT };
            let name = &self.content.goods[&swing.good].name;
            self.log(format!("{label}: {name}"));
        }
        self.world.markets.apply_shock(
            swing.good,
            Some(swing.city),
            swing.factor,
            Some(swing.days),
            date,
        );
    }

    /// Сделки челноков на толкучке, где стоит игрок, — строки в ленте: «Привоз: Джинсы (фирма),
    /// 4 шт — Наташа Турецкая».
    pub(super) fn log_rival_deals(&mut self, deals: &[RivalDeal]) {
        let messages: Vec<String> = deals
            .iter()
            .filter(|deal| self.is_at_market_of(&deal.city))
            .map(|deal| {
                let rival = &self.content.npcs[&deal.rival].name;
                let good = &self.content.goods[&deal.good];
                let verb = match deal.side {
                    DealSide::Bought => BOUGHT_UP,
                    DealSide::Sold => BROUGHT,
                };
                format!(
                    "{verb}: {}, {} {} — {rival}",
                    good.name, deal.quantity, good.unit
                )
            })
            .collect();
        for message in messages {
            self.log(message);
        }
    }

    /// Игрок стоит на толкучке этого города: не в пути и не в зоне.
    fn is_at_market_of(&self, city: &CityId) -> bool {
        self.world.journey.is_none() && self.world.zone.is_none() && &self.world.city == city
    }
}
