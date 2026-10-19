use serde::{Deserialize, Serialize};

use crate::time::GameDate;

/// Можно ли сейчас обратиться к человеку.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Availability {
    Available,
    /// Уехал, лёг в больницу, сидит — до этой даты.
    AwayUntil(GameDate),
    /// Умер, уехал насовсем, отвернулся навсегда.
    Lost,
}
