//! Время мира: календарь от 1 января 1984 года, эпохи по датам и часы со скоростями и паузой.

mod clock;
mod epoch_table;
mod game_date;
mod time_rules;

pub use clock::{Clock, ClockSettings, DayTick, FastForward, InvalidSpeed, MAX_SPEED};
pub use epoch_table::{Epoch, EpochTable};
pub use game_date::GameDate;
pub use time_rules::{TimeRules, TimeRulesError};
