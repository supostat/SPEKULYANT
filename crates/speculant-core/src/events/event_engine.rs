use std::collections::{BTreeMap, BTreeSet};

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::Rng;

use super::{JournalEntry, PendingEvent, Resolution, ScheduledEvent, Situation};
use crate::content::{EventId, FlagId, RoadEvent};
use crate::time::GameDate;

/// Сколько старших бит случайного числа даёт равномерную долю от нуля до единицы.
const FRACTION_BITS: u32 = 53;

/// Память событий одной жизни: флаги прошлых решений, когда что случалось, отложенные
/// продолжения, событие, остановившее время, и журнал.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EventEngine {
    flags: BTreeSet<FlagId>,
    last_happened: BTreeMap<EventId, GameDate>,
    scheduled: Vec<ScheduledEvent>,
    pending: Option<PendingEvent>,
    journal: Vec<JournalEntry>,
}

impl EventEngine {
    pub fn flags(&self) -> &BTreeSet<FlagId> {
        &self.flags
    }

    pub fn pending(&self) -> Option<&PendingEvent> {
        self.pending.as_ref()
    }

    pub fn scheduled(&self) -> &[ScheduledEvent] {
        &self.scheduled
    }

    pub fn journal(&self) -> &[JournalEntry] {
        &self.journal
    }

    /// Событие готово прийти сегодня: оно подходит к положению дел, однократное ещё не случалось,
    /// а повторяемое отдохнуло свой срок.
    pub fn is_ready(&self, event: &EventId, situation: &Situation<'_>) -> bool {
        let content = situation.content;
        let data = &content.events[event];
        let rested = match self.last_happened.get(event) {
            None => true,
            Some(_) if data.once => false,
            Some(last) => {
                let rest = data
                    .cooldown_days
                    .unwrap_or(content.event_rules.cooldown_days);
                situation.today >= last.after_days(rest)
            }
        };
        rested && situation.allows_event(data)
    }

    /// Готовые сегодня события с их весами — по порядку идентификаторов.
    pub fn ready_events(&self, situation: &Situation<'_>) -> Vec<(EventId, u32)> {
        situation
            .content
            .events
            .iter()
            .filter(|(id, _)| self.is_ready(id, situation))
            .map(|(id, event)| (id.clone(), event.weight))
            .collect()
    }

    /// Готовые сегодня дорожные события маршрута с весами маршрута.
    pub fn ready_road_events(
        &self,
        situation: &Situation<'_>,
        road_events: &[RoadEvent],
    ) -> Vec<(EventId, u32)> {
        road_events
            .iter()
            .filter(|road| self.is_ready(&road.event, situation))
            .map(|road| (road.event.clone(), road.weight))
            .collect()
    }

    /// Случилось ли то, что случается с вероятностью `probability`.
    pub fn roll(rng: &mut ChaCha8Rng, probability: f64) -> bool {
        let fraction = (rng.next_u64() >> (u64::BITS - FRACTION_BITS)) as f64
            / (1_u64 << FRACTION_BITS) as f64;
        fraction < probability
    }

    /// Одно событие из готовых, с вероятностью по весу; `None`, если готовых нет.
    pub fn pick(rng: &mut ChaCha8Rng, weighted: &[(EventId, u32)]) -> Option<EventId> {
        let total: u64 = weighted.iter().map(|(_, weight)| u64::from(*weight)).sum();
        if total == 0 {
            return None;
        }
        let mut ticket = rng.next_u64() % total;
        for (event, weight) in weighted {
            let weight = u64::from(*weight);
            if ticket < weight {
                return Some(event.clone());
            }
            ticket -= weight;
        }
        unreachable!("билет меньше суммы весов попадает в одно из событий")
    }

    /// Продолжения, срок которых пришёл, в порядке назначения; они уходят из расписания.
    pub fn take_due(&mut self, today: GameDate) -> Vec<EventId> {
        let (due, later): (Vec<_>, Vec<_>) = self
            .scheduled
            .drain(..)
            .partition(|scheduled| scheduled.due <= today);
        self.scheduled = later;
        due.into_iter().map(|scheduled| scheduled.event).collect()
    }

    pub fn schedule(&mut self, event: EventId, due: GameDate) {
        self.scheduled.push(ScheduledEvent { event, due });
    }

    /// Событие случилось и остановило время.
    pub fn start(&mut self, event: EventId, today: GameDate) {
        assert!(
            self.pending.is_none(),
            "новое событие приходит только после того, как прочитан исход прошлого"
        );
        self.last_happened.insert(event.clone(), today);
        self.journal.push(JournalEntry {
            date: today,
            event: event.clone(),
            choice: None,
        });
        self.pending = Some(PendingEvent {
            event,
            resolution: None,
        });
    }

    /// Выбор сделан, последствия применены: событие показывает исход.
    pub fn resolve(&mut self, resolution: Resolution) {
        let pending = self
            .pending
            .as_mut()
            .expect("выбирают только в ожидающем событии");
        let entry = self
            .journal
            .last_mut()
            .expect("ожидающее событие записано в журнал");
        entry.choice = Some(resolution.choice);
        pending.resolution = Some(resolution);
    }

    /// Исход прочитан: время идёт дальше.
    pub fn close(&mut self) {
        self.pending = None;
    }

    pub fn set_flag(&mut self, flag: FlagId) {
        self.flags.insert(flag);
    }

    pub fn clear_flag(&mut self, flag: &FlagId) {
        self.flags.remove(flag);
    }
}
