//! Метапрогрессия (§11.3): кончившаяся жизнь оставляет следующим открытые биографии, записи в
//! книжке, увиденную историю и строку в рекордах.

use std::sync::Arc;

use super::Game;
use crate::life::LifeSummary;
use crate::meta::Record;

impl Game {
    /// Итоги жизни переходят в метапрогрессию: открытые биографии; цены сделок, лучшая сделка и
    /// последнее, что жизнь знала о каждом знакомом и сопернике, — в книжку; исторические узлы,
    /// случившиеся до финала, — в известную историю; жизнь — в таблицу рекордов своего режима.
    pub(super) fn remember_life(&mut self, summary: &LifeSummary) {
        let content = Arc::clone(&self.content);
        let ended = summary.ended;
        let world = &self.world;
        self.meta.unlock(&summary.discoveries);
        let notebook = self.meta.notebook_mut();
        notebook.note_prices(world.chronicle.prices());
        if let Some(deal) = &summary.best_deal {
            notebook.note_deal(deal);
        }
        let people = world
            .contacts
            .values()
            .chain(world.rivals.iter().map(|rival| rival.memory()));
        for person in people {
            if let Some(last) = person.history().last() {
                notebook.note_person(person.npc().clone(), last.clone());
            }
        }
        let witnessed = world
            .timeline
            .nodes()
            .iter()
            .filter(|node| content.history.contains_key(&node.node) && node.date <= ended);
        for node in witnessed {
            self.meta.witness(node.node.clone(), node.date.year());
        }
        self.meta.records_mut().add(
            Record {
                name: world.character.name().to_owned(),
                biography: world.character.biography().clone(),
                capital: summary.capital,
                ended,
                ending: summary.ending.clone(),
            },
            world.ironman,
            content.life.records_size,
        );
    }
}
