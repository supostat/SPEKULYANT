//! Ход событий в жизни: событие дня, продолжения в срок, выбор и прочитанный исход.

use std::sync::Arc;

use super::Game;
use crate::command_error::CommandError;
use crate::content::EventId;
use crate::events::{EventEngine, Resolution};
use crate::time::GameDate;

const EVENTS_STREAM: &str = "events";

impl Game {
    /// Начинает событие сейчас, если оно готово: так приходят продолжения цепочек, звонки
    /// знакомым и события дороги.
    pub fn start_event(&mut self, event: &EventId) -> Result<(), CommandError> {
        if !self.content.events.contains_key(event) {
            return Err(CommandError::UnknownEvent {
                event: event.to_string(),
            });
        }
        if self.events.pending().is_some() {
            return Err(CommandError::EventPending);
        }
        let today = self.today();
        if !self.events.is_ready(event, &self.situation(today)) {
            return Err(CommandError::EventNotReady {
                event: event.to_string(),
            });
        }
        self.events.start(event.clone(), today);
        Ok(())
    }

    /// Событие дня; `true`, если оно случилось. Сначала — продолжения, чей срок пришёл: первое
    /// готовое начинается, остальные готовые ждут завтрашнего дня, а неготовые пропадают —
    /// момент упущен. Если продолжений нет, день разыгрывается: с шансом правил случается одно
    /// из готовых событий по весам.
    pub(super) fn start_day_event(&mut self, date: GameDate) -> bool {
        let mut started = false;
        for event in self.events.take_due(date) {
            if !self.events.is_ready(&event, &self.situation(date)) {
                continue;
            }
            if started {
                self.events.schedule(event, date.next());
            } else {
                self.events.start(event, date);
                started = true;
            }
        }
        if started {
            return true;
        }
        let daily_chance = self.content.event_rules.daily_chance;
        if !EventEngine::roll(self.rng.stream(EVENTS_STREAM), daily_chance) {
            return false;
        }
        let ready = self.events.ready_events(&self.situation(date));
        match EventEngine::pick(self.rng.stream(EVENTS_STREAM), &ready) {
            Some(event) => {
                self.events.start(event, date);
                true
            }
            None => false,
        }
    }

    /// Выбор `number` (от 1) в ожидающем событии: последствия применяются сразу, продолжения
    /// назначаются с их вероятностью, событие показывает исход.
    pub(super) fn choose(&mut self, number: usize) -> Result<(), CommandError> {
        let pending = self.events.pending().ok_or(CommandError::NoPendingEvent)?;
        if pending.resolution.is_some() {
            return Err(CommandError::ChoiceAlreadyMade);
        }
        let content = Arc::clone(&self.content);
        let event = &content.events[&pending.event];
        let count = event.choices.len();
        let index = number.checked_sub(1).filter(|index| *index < count).ok_or(
            CommandError::NoSuchChoice {
                choice: number,
                count,
            },
        )?;
        let choice = &event.choices[index];
        let today = self.today();
        if !self.situation(today).allows_choice(choice) {
            return Err(CommandError::ChoiceUnavailable { choice: number });
        }
        let effects = self.apply_consequences(&choice.consequences, &event.title, today);
        for follow_up in &choice.follow_ups {
            if EventEngine::roll(self.rng.stream(EVENTS_STREAM), follow_up.chance) {
                self.events.schedule(
                    follow_up.event.clone(),
                    today.after_days(follow_up.after_days),
                );
            }
        }
        self.events.resolve(Resolution {
            choice: index,
            effects,
        });
        Ok(())
    }

    /// Исход прочитан — событие закрывается, время идёт.
    pub(super) fn close_event(&mut self) -> Result<(), CommandError> {
        let pending = self.events.pending().ok_or(CommandError::NoPendingEvent)?;
        if pending.resolution.is_none() {
            return Err(CommandError::ChoiceRequired);
        }
        self.events.close();
        Ok(())
    }
}
