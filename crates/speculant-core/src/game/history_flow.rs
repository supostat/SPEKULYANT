//! История в жизни (§3.6, решение 15.9): узел таймлайна в свой день пишет новость, сдвигает цены,
//! ставит флаги исхода и начинает своё событие.

use std::sync::Arc;

use super::Game;
use crate::time::GameDate;

impl Game {
    /// Узлы истории этого дня: новость исхода — в ленту, шоки цен — с силой, возведённой в масштаб
    /// узла, флаги исхода — в память событий, событие исхода — в продолжения этого дня: оно
    /// начнётся, если готово, — дома, а в зоне только событие зоны, как амнистия.
    pub(super) fn live_history(&mut self, date: GameDate) {
        let content = Arc::clone(&self.content);
        let nodes: Vec<_> = self.world.timeline.on(date).cloned().collect();
        for node in nodes {
            let outcome = node.outcome(&content);
            self.page(outcome.news.clone(), false);
            self.log(outcome.news.clone());
            for shock in &outcome.price_shocks {
                self.world.markets.apply_shock(
                    shock.good.clone(),
                    shock.city.clone(),
                    shock.factor.powf(node.scale),
                    shock.days,
                    date,
                );
            }
            for flag in &outcome.set_flags {
                self.world.events.set_flag(flag.clone());
            }
            if let Some(event) = &outcome.event {
                self.world.events.schedule(event.clone(), date);
            }
        }
    }
}
