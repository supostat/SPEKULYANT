//! Звонок связи (§9, §13.2): человек на связи начинает с игроком готовое событие о себе — по
//! весам, как событие дня; если такого события сейчас нет, он не берёт трубку.

use super::Game;
use super::event_flow::EVENTS_STREAM;
use crate::command_error::CommandError;
use crate::content::{EventId, NpcId};
use crate::events::EventEngine;
use crate::reputation::Availability;

const NO_ANSWER: &str = "не берёт трубку";

impl Game {
    pub(super) fn call(&mut self, contact: &str) -> Result<(), CommandError> {
        let npc = NpcId::new(contact);
        let today = self.today();
        let known = self
            .world
            .contacts
            .get(&npc)
            .ok_or_else(|| CommandError::NotAContact {
                contact: contact.to_owned(),
            })?;
        let name = self.content.npcs[&npc].name.clone();
        match known.availability() {
            Availability::Lost => return Err(CommandError::ContactLost { name }),
            Availability::AwayUntil(until) if today < until => {
                return Err(CommandError::ContactAway {
                    name,
                    until: until.numeric(),
                });
            }
            _ => {}
        }
        let ready: Vec<(EventId, u32)> = self
            .world
            .events
            .ready_events(&self.situation(today))
            .into_iter()
            .filter(|(event, _)| self.content.events[event].conditions.npc.as_ref() == Some(&npc))
            .collect();
        match EventEngine::pick(self.world.rng.stream(EVENTS_STREAM), &ready) {
            Some(event) => self.begin_event(event, today, None),
            None => self.log(format!("{name} {NO_ANSWER}")),
        }
        Ok(())
    }
}
