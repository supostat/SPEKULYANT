use crate::content::{Content, HistoryOutcome, TimelineNodeId};
use crate::time::GameDate;

/// Узел таймлайна мира: какой исторический узел или шаблон генератора, в какой день, с каким
/// масштабом и каким исходом — номер среди исходов узла.
#[derive(Clone, Debug, PartialEq)]
pub struct TimelineNode {
    pub node: TimelineNodeId,
    pub date: GameDate,
    pub scale: f64,
    pub outcome: usize,
}

impl TimelineNode {
    /// Исход узла из данных: исторического узла или шаблона генератора.
    pub fn outcome<'c>(&self, content: &'c Content) -> &'c HistoryOutcome {
        let outcomes = match content.history.get(&self.node) {
            Some(node) => &node.outcomes,
            None => &content.history_templates[&self.node].outcomes,
        };
        &outcomes[self.outcome]
    }
}
