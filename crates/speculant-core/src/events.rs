//! Система событий (§10): условия над положением дел, взвешенный розыгрыш дня с однократными и
//! отдыхающими событиями, выборы, отложенные продолжения, подстановки текста и журнал жизни.

mod event_engine;
mod event_rules;
mod journal_entry;
mod pending_event;
mod resolution;
mod scheduled_event;
mod situation;
mod substitutions;

pub use event_engine::EventEngine;
pub use event_rules::EventRules;
pub use journal_entry::JournalEntry;
pub use pending_event::PendingEvent;
pub use resolution::Resolution;
pub use scheduled_event::ScheduledEvent;
pub use situation::Situation;
pub use substitutions::Substitutions;
