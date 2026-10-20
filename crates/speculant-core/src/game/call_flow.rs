//! Звонок связи (§9, §13.2): человек на связи начинает с игроком готовое событие о себе — по
//! весам, как событие дня; если такого события сейчас нет, знакомый из другого города за услугу
//! диктует цены своей толкучки, а остальные не берут трубку.

use super::Game;
use super::event_flow::EVENTS_STREAM;
use crate::command_error::CommandError;
use crate::content::{EventId, NpcId};
use crate::events::EventEngine;
use crate::reputation::Availability;
use crate::time::GameDate;

const NO_ANSWER: &str = "не берёт трубку";
const PRICES_BY_PHONE: &str = "Цены по телефону";

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
            None => self.hear_prices(&npc, today),
        }
        Ok(())
    }

    /// Знакомый из другого города без своего повода диктует цены своей толкучки — за услугу: они
    /// ложатся в цены по городам, а в ленте строка. Знакомый без города или из города, где игрок,
    /// не берёт трубку; тот, кто услуги не окажет, говорит почему.
    fn hear_prices(&mut self, npc: &NpcId, today: GameDate) {
        let template = &self.content.npcs[npc];
        let name = template.name.clone();
        let Some(city) = template
            .city
            .clone()
            .filter(|city| *city != self.world.city)
        else {
            self.log(format!("{name} {NO_ANSWER}"));
            return;
        };
        let favour = format!("{PRICES_BY_PHONE}: {}", self.content.cities[&city].name);
        let contact = self
            .world
            .contacts
            .get_mut(npc)
            .expect("связь найдена при звонке");
        match contact.ask(self.content.reputation, today, &favour) {
            Ok(()) => {
                self.remember_market(city, today);
                self.log(format!("{favour} — {name}"));
            }
            Err(refusal) => self.log(format!("{name}: {refusal}")),
        }
    }
}
