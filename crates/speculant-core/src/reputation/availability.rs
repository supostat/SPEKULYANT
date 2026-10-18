use crate::time::GameDate;

/// Можно ли сейчас обратиться к человеку.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Availability {
    Available,
    /// Уехал, лёг в больницу, сидит — до этой даты.
    AwayUntil(GameDate),
    /// Умер, уехал насовсем, отвернулся навсегда.
    Lost,
}
