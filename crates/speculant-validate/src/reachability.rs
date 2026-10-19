use std::collections::BTreeMap;

use speculant_core::Content;
use speculant_core::content::{Event, EventId, FlagId};
use speculant_core::time::GameDate;

use crate::days::Days;

/// Что событию нужно от мира сверх календаря: флаг из условия `flags_set` и, если это событие
/// зоны, — сидеть в зоне.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Source {
    Flag(FlagId),
    Zone,
}

/// Почему событие с живым календарём всё же не случится.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Blocker {
    /// Источника нет ни в одной жизни.
    Missing(Source),
    /// Источник появляется позже последнего дня события.
    Late { source: Source, earliest: GameDate },
}

/// Достижимость событий над календарём: самый ранний день каждого события, которое может
/// случиться, и каждого источника, который в какой-то жизни появляется. Флаг ставят исходы узлов
/// и шаблонов и выборы достижимых событий; в зону сажает выбор достижимого события со сроком, и с
/// того же дня возможен выход из неё с флагом выхода.
pub struct Reachability {
    events: BTreeMap<EventId, GameDate>,
    sources: BTreeMap<Source, GameDate>,
}

impl Reachability {
    pub fn compute(content: &Content, days: &BTreeMap<EventId, Days>) -> Self {
        let mut reachability = Self {
            events: BTreeMap::new(),
            sources: BTreeMap::new(),
        };
        for node in content.history.values() {
            for flag in node.outcomes.iter().flat_map(|outcome| &outcome.set_flags) {
                reachability.lower(Source::Flag(flag.clone()), node.window.from);
            }
        }
        for template in content.history_templates.values() {
            for flag in template
                .outcomes
                .iter()
                .flat_map(|outcome| &outcome.set_flags)
            {
                reachability.lower(Source::Flag(flag.clone()), content.history_generator.from);
            }
        }
        while reachability.relax(content, days) {}
        reachability
    }

    /// Самый ранний день события, если оно случается хоть в одной жизни.
    pub fn earliest(&self, event: &EventId) -> Option<GameDate> {
        self.events.get(event).copied()
    }

    /// Почему событие не случится: источник, которого нет, или самый поздний из тех, что
    /// приходят после последнего дня события.
    pub fn blocker(&self, event: &Event, days: &Days) -> Blocker {
        let mut late = Vec::new();
        for source in needs(event) {
            match self.sources.get(&source) {
                None => return Blocker::Missing(source),
                Some(earliest) => late.push((*earliest, source)),
            }
        }
        let (earliest, source) = late
            .into_iter()
            .filter(|(earliest, _)| days.since(*earliest).is_empty())
            .max()
            .expect("событие без живого источника недостижимо из-за позднего источника");
        Blocker::Late { source, earliest }
    }

    /// Один проход по событиям; `true`, если чей-то самый ранний день стал раньше.
    fn relax(&mut self, content: &Content, days: &BTreeMap<EventId, Days>) -> bool {
        let mut changed = false;
        for (id, event_days) in days {
            let event = &content.events[id];
            let Some(earliest) = self.earliest_day(event, event_days) else {
                continue;
            };
            if self.events.get(id).is_some_and(|known| *known <= earliest) {
                continue;
            }
            self.events.insert(id.clone(), earliest);
            changed = true;
            for choice in &event.choices {
                for flag in &choice.consequences.set_flags {
                    self.lower(Source::Flag(flag.clone()), earliest);
                }
                if choice.consequences.sentence_months.is_some() {
                    self.lower(Source::Zone, earliest);
                    self.lower(Source::Flag(content.zone.release_flag.clone()), earliest);
                }
            }
        }
        changed
    }

    fn earliest_day(&self, event: &Event, days: &Days) -> Option<GameDate> {
        let mut from = GameDate::START;
        for source in needs(event) {
            from = from.max(*self.sources.get(&source)?);
        }
        days.since(from).first()
    }

    fn lower(&mut self, source: Source, day: GameDate) {
        let earliest = self.sources.entry(source).or_insert(day);
        *earliest = (*earliest).min(day);
    }
}

fn needs(event: &Event) -> impl Iterator<Item = Source> {
    let zone = event.epoch.is_none().then_some(Source::Zone);
    event
        .conditions
        .flags_set
        .iter()
        .cloned()
        .map(Source::Flag)
        .chain(zone)
}
