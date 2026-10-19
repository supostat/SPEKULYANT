//! Ход событий в жизни: событие дня, продолжения в срок, выбор и прочитанный исход.

use std::sync::Arc;

use rand_chacha::rand_core::Rng;

use super::Game;
use crate::business::{BusinessId, Incident};
use crate::command_error::CommandError;
use crate::content::EventId;
use crate::events::{EventEngine, PendingEvent, Resolution, Substitutions};
use crate::time::GameDate;
use crate::view::Screen;

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
        if self.world.events.pending().is_some() {
            return Err(CommandError::EventPending);
        }
        let today = self.today();
        if !self.world.events.is_ready(event, &self.situation(today)) {
            return Err(CommandError::EventNotReady {
                event: event.to_string(),
            });
        }
        self.begin_event(event.clone(), today, None);
        Ok(())
    }

    /// Событие дня; `true`, если оно случилось. Сначала — продолжения, чей срок пришёл; если их
    /// нет, день разыгрывается с шансом правил событий.
    pub(super) fn start_day_event(&mut self, date: GameDate) -> bool {
        let daily_chance = self.content.event_rules.daily_chance;
        self.start_due_follow_up(date) || self.draw_event(date, daily_chance)
    }

    /// Продолжения, чей срок пришёл; `true`, если одно началось. Первое готовое начинается,
    /// остальные готовые ждут завтрашнего дня, а неготовые пропадают — момент упущен.
    pub(super) fn start_due_follow_up(&mut self, date: GameDate) -> bool {
        let mut started = false;
        for event in self.world.events.take_due(date) {
            if !self.world.events.is_ready(&event, &self.situation(date)) {
                continue;
            }
            if started {
                self.world.events.schedule(event, date.next());
            } else {
                self.begin_event(event, date, None);
                started = true;
            }
        }
        started
    }

    /// Розыгрыш: с шансом `chance` случается одно из готовых событий по весам; `true`, если
    /// случилось.
    pub(super) fn draw_event(&mut self, date: GameDate, chance: f64) -> bool {
        if !EventEngine::roll(self.world.rng.stream(EVENTS_STREAM), chance) {
            return false;
        }
        let ready = self.world.events.ready_events(&self.situation(date));
        match EventEngine::pick(self.world.rng.stream(EVENTS_STREAM), &ready) {
            Some(event) => {
                self.begin_event(event, date, None);
                true
            }
            None => false,
        }
    }

    /// Событие происшествия с делом, которое вышло за политики; `true`, если оно случилось. Из
    /// готовых событий про это происшествие сид выбирает одно по весам; готовых нет — происшествие
    /// проходит без последствий.
    pub(super) fn start_incident_event(
        &mut self,
        business: &BusinessId,
        incident: Incident,
        date: GameDate,
    ) -> bool {
        let ready = self
            .world
            .events
            .ready_incident_events(&self.situation(date), incident);
        match EventEngine::pick(self.world.rng.stream(EVENTS_STREAM), &ready) {
            Some(event) => {
                self.begin_event(event, date, Some(business.clone()));
                true
            }
            None => false,
        }
    }

    /// Начинает готовое событие; если в нём участвует соперник, сид выбирает одного из
    /// подходящих. Подстановки снимаются сейчас — с участниками и делом этого дня.
    pub(super) fn begin_event(
        &mut self,
        event: EventId,
        today: GameDate,
        business: Option<BusinessId>,
    ) {
        let rival = self.content.events[&event]
            .conditions
            .rival
            .and_then(|condition| {
                let matching = self.world.rivals.matching(&condition, today);
                let count = u64::try_from(matching.len()).expect("соперников меньше 2^64");
                (count > 0).then(|| {
                    let index = self.world.rng.stream(EVENTS_STREAM).next_u64() % count;
                    matching[usize::try_from(index).expect("номер меньше числа соперников")].clone()
                })
            });
        let substitutions = Substitutions::for_event(
            &self.content.events[&event],
            rival.as_ref(),
            business.as_ref().map(|id| &self.world.businesses[id]),
            &self.situation(today),
            &self.world.markets,
        );
        self.world.events.start(
            PendingEvent {
                event,
                rival,
                business,
                substitutions,
                resolution: None,
            },
            today,
        );
    }

    /// Выбор `number` (от 1) в ожидающем событии: последствия применяются сразу, продолжения
    /// назначаются с их вероятностью, событие показывает исход.
    pub(super) fn choose(&mut self, number: usize) -> Result<(), CommandError> {
        let pending = self
            .world
            .events
            .pending()
            .ok_or(CommandError::NoPendingEvent)?;
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
        let rival = pending.rival.clone();
        let business = pending.business.clone();
        let effects = self.apply_consequences(
            &choice.consequences,
            &event.title,
            rival.as_ref(),
            business.as_ref(),
            today,
        );
        for follow_up in &choice.follow_ups {
            if EventEngine::roll(self.world.rng.stream(EVENTS_STREAM), follow_up.chance) {
                self.world.events.schedule(
                    follow_up.event.clone(),
                    today.after_days(follow_up.after_days),
                );
            }
        }
        self.world.events.resolve(Resolution {
            choice: index,
            effects,
        });
        Ok(())
    }

    /// Исход прочитан — событие закрывается, время идёт; случилось оно в последний день пути —
    /// игрок наконец приезжает. Если исход кончил жизнь, открываются итоги; в зоне — экран зоны,
    /// а если исход выпустил на волю — рынок.
    pub(super) fn close_event(&mut self) -> Result<(), CommandError> {
        let pending = self
            .world
            .events
            .pending()
            .ok_or(CommandError::NoPendingEvent)?;
        if pending.resolution.is_none() {
            return Err(CommandError::ChoiceRequired);
        }
        self.world.events.close();
        if self.world.summary.is_some() {
            self.world.screen = Screen::Summary;
            return Ok(());
        }
        if self.world.zone.is_some() {
            self.world.screen = Screen::Zone;
            return Ok(());
        }
        if self.world.screen == Screen::Zone {
            self.world.screen = Screen::Market;
        }
        self.arrive_if_due(self.today());
        Ok(())
    }
}
