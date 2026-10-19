use std::collections::BTreeMap;

use speculant_core::Content;
use speculant_core::content::{Conditions, DateWindow, Event, TimelineNodeId};
use speculant_core::time::GameDate;

use crate::days::{DaySpan, Days};

/// Календарь условий: в какие дни событие может случиться по эпохе своего каталога, условию
/// `epochs`, окну дат и узлу таймлайна.
pub struct Calendar<'a> {
    content: &'a Content,
    epochs: BTreeMap<&'a str, Days>,
}

/// Условие, после которого у события не осталось ни одного дня, и дни, что были до него.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Narrowing {
    Epochs {
        before: Days,
    },
    Window {
        window: DateWindow,
        before: Days,
    },
    Timeline {
        node: TimelineNodeId,
        earliest: GameDate,
        before: Days,
    },
}

impl Narrowing {
    /// Начало строки условия в файле события — по нему ищется строка.
    pub fn needle(&self) -> &'static str {
        match self {
            Self::Epochs { .. } => "epochs =",
            Self::Window { .. } => "window =",
            Self::Timeline { .. } => "timeline =",
        }
    }

    pub fn message(&self) -> String {
        match self {
            Self::Epochs { before } => {
                format!("условие epochs не пересекает его дни ({before})")
            }
            Self::Window { window, before } => {
                format!(
                    "окно {} не пересекает его дни ({before})",
                    window_days(*window)
                )
            }
            Self::Timeline {
                node,
                earliest,
                before,
            } => format!(
                "узел {node} случается не раньше {}, а дни события — {before}",
                earliest.to_iso()
            ),
        }
    }
}

impl<'a> Calendar<'a> {
    pub fn new(content: &'a Content) -> Self {
        let listed = content.time.epochs.epochs();
        let epochs = listed
            .iter()
            .enumerate()
            .map(|(index, epoch)| {
                let until = listed.get(index + 1).map(|next| next.starts);
                (
                    epoch.id.as_str(),
                    Days::from_spans([DaySpan::new(epoch.starts, until)]),
                )
            })
            .collect();
        Self { content, epochs }
    }

    /// Дни события: событие эпохи живёт в её днях, событие зоны — в любые; дальше их сужают
    /// условия события.
    pub fn event_days(&self, event: &Event) -> Result<Days, Narrowing> {
        let own = match &event.epoch {
            Some(epoch) => self.epochs[epoch.as_str()].clone(),
            None => Days::all(),
        };
        self.narrow(own, &event.conditions)
    }

    /// Дни, которые оставляют условия из уже известных дней.
    pub fn narrow(&self, days: Days, conditions: &Conditions) -> Result<Days, Narrowing> {
        let mut days = days;
        if !conditions.epochs.is_empty() {
            let listed = Days::from_spans(
                conditions
                    .epochs
                    .iter()
                    .flat_map(|epoch| self.epochs[epoch.as_str()].spans().iter().copied()),
            );
            let narrowed = days.intersect(&listed);
            if narrowed.is_empty() {
                return Err(Narrowing::Epochs { before: days });
            }
            days = narrowed;
        }
        if let Some(window) = conditions.window {
            let narrowed =
                days.intersect(&Days::from_spans([DaySpan::new(window.from, window.until)]));
            if narrowed.is_empty() {
                return Err(Narrowing::Window {
                    window,
                    before: days,
                });
            }
            days = narrowed;
        }
        if let Some(node) = &conditions.timeline {
            let earliest = self.node_earliest(node);
            let narrowed = days.since(earliest);
            if narrowed.is_empty() {
                return Err(Narrowing::Timeline {
                    node: node.clone(),
                    earliest,
                    before: days,
                });
            }
            days = narrowed;
        }
        Ok(days)
    }

    /// Первый день, когда узел мог случиться: начало окна исторического узла или начало
    /// генератора для шаблона песочницы.
    pub fn node_earliest(&self, node: &TimelineNodeId) -> GameDate {
        self.content
            .history
            .get(node)
            .map_or(self.content.history_generator.from, |history| {
                history.window.from
            })
    }
}

fn window_days(window: DateWindow) -> Days {
    Days::from_spans([DaySpan::new(window.from, window.until)])
}
