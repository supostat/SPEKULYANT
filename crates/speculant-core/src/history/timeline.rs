use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::Rng;

use super::{DayRange, TimelineNode};
use crate::content::{Content, HistoryOutcome, HistoryWindow, ScaleRange, TimelineNodeId};
use crate::events::EventEngine;
use crate::time::GameDate;

/// Таймлайн мира (§3.6, решение 15.9), зафиксированный по сиду при создании мира: исторические
/// узлы в своих окнах и узлы генератора песочницы — по дням.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    nodes: Vec<TimelineNode>,
}

impl Timeline {
    /// История мира. Каждый исторический узел получает день в окне, масштаб из разброса и исход по
    /// весам. Затем генератор от своего начала до горизонта шагает промежутками из правил и на
    /// каждом шаге ставит по весам один из шаблонов, отдохнувших свой срок; если все отдыхают,
    /// шаг проходит пустым. Узлы одного дня идут по порядку идентификаторов.
    pub fn generate(content: &Content, rng: &mut ChaCha8Rng) -> Self {
        let mut nodes: Vec<TimelineNode> = content
            .history
            .iter()
            .map(|(id, node)| TimelineNode {
                node: id.clone(),
                date: day_in(node.window, rng),
                scale: scale_in(node.scale, rng),
                outcome: outcome_of(&node.outcomes, rng),
            })
            .collect();
        nodes.extend(generated(content, rng));
        nodes.sort_by(|left, right| (left.date, &left.node).cmp(&(right.date, &right.node)));
        Self { nodes }
    }

    pub fn nodes(&self) -> &[TimelineNode] {
        &self.nodes
    }

    /// Узлы этого дня.
    pub fn on(&self, date: GameDate) -> impl Iterator<Item = &TimelineNode> {
        self.nodes.iter().filter(move |node| node.date == date)
    }

    /// Узел случился к этому дню; узел генератора — хотя бы раз.
    pub fn has_happened(&self, node: &TimelineNodeId, today: GameDate) -> bool {
        self.nodes
            .iter()
            .any(|happened| &happened.node == node && happened.date <= today)
    }
}

/// Узлы генератора песочницы от его начала до горизонта.
fn generated(content: &Content, rng: &mut ChaCha8Rng) -> Vec<TimelineNode> {
    let generator = content.history_generator;
    let templates = &content.history_templates;
    let mut last: BTreeMap<&TimelineNodeId, GameDate> = BTreeMap::new();
    let mut nodes = Vec::new();
    let mut date = generator.from;
    loop {
        date = date.after_days(days_in(generator.gap_days, rng));
        if date >= generator.until {
            return nodes;
        }
        let rested: Vec<(&TimelineNodeId, u32)> = templates
            .iter()
            .filter(|(id, template)| {
                last.get(id)
                    .is_none_or(|previous| date >= previous.after_days(template.rest_days))
            })
            .map(|(id, template)| (id, template.weight))
            .collect();
        let Some(id) = EventEngine::pick(rng, &rested) else {
            continue;
        };
        let template = &templates[id];
        last.insert(id, date);
        nodes.push(TimelineNode {
            node: id.clone(),
            date,
            scale: scale_in(template.scale, rng),
            outcome: outcome_of(&template.outcomes, rng),
        });
    }
}

fn day_in(window: HistoryWindow, rng: &mut ChaCha8Rng) -> GameDate {
    let days = u64::try_from(window.until.day_index() - window.from.day_index())
        .expect("окно узла не пустое — это проверяет загрузка");
    let offset = u32::try_from(rng.next_u64() % days).expect("окно узла короче 2^32 дней");
    window.from.after_days(offset)
}

fn days_in(range: DayRange, rng: &mut ChaCha8Rng) -> u32 {
    let spread = u64::from(range.to - range.from) + 1;
    range.from + u32::try_from(rng.next_u64() % spread).expect("разброс дней мал")
}

fn scale_in(scale: ScaleRange, rng: &mut ChaCha8Rng) -> f64 {
    scale.from + EventEngine::fraction(rng) * (scale.to - scale.from)
}

fn outcome_of(outcomes: &[HistoryOutcome], rng: &mut ChaCha8Rng) -> usize {
    let weighted: Vec<(usize, u32)> = outcomes
        .iter()
        .enumerate()
        .map(|(index, outcome)| (index, outcome.weight))
        .collect();
    EventEngine::pick(rng, &weighted).expect("у узла есть исход с весом — это проверяет загрузка")
}
