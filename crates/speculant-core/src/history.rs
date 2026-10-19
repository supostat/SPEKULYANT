//! История (§3, решение 15.9): таймлайн по сиду — исторические узлы в окнах дат с масштабом и
//! исходом, а после 2004 года — узлы генератора из шаблонов с весами и сроками без повторов.

mod history_generator;
mod timeline;
mod timeline_node;

pub use history_generator::{DayRange, HistoryGenerator};
pub use timeline::Timeline;
pub use timeline_node::TimelineNode;
