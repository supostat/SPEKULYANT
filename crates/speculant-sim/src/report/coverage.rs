use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use speculant_core::Content;
use speculant_core::content::EventId;

use crate::LifeOutcome;

/// Покрытие событий прогоном — то, что валидатор сверяет с данными: сколько раз сработало каждое
/// событие данных, и на какой жизни сработало последнее из впервые сработавших.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Coverage {
    pub seed: u64,
    pub lives: usize,
    /// Сколько первых жизней прогона дали всё покрытие, которое дал прогон.
    pub saturated_at: usize,
    pub events: BTreeMap<EventId, usize>,
}

impl Coverage {
    pub(crate) fn build(content: &Content, seed: u64, outcomes: &[LifeOutcome]) -> Self {
        let mut events: BTreeMap<EventId, usize> = content
            .events
            .keys()
            .map(|event| (event.clone(), 0))
            .collect();
        let mut saturated_at = 0;
        for (index, outcome) in outcomes.iter().enumerate() {
            for (_, event) in &outcome.events {
                let count = events
                    .get_mut(event)
                    .expect("журнал пишет только события данных");
                if *count == 0 {
                    saturated_at = index + 1;
                }
                *count += 1;
            }
        }
        Self {
            seed,
            lives: outcomes.len(),
            saturated_at,
            events,
        }
    }

    /// События, которые ни разу не сработали.
    pub fn never_fired(&self) -> impl Iterator<Item = &EventId> {
        self.events
            .iter()
            .filter(|(_, count)| **count == 0)
            .map(|(event, _)| event)
    }
}
