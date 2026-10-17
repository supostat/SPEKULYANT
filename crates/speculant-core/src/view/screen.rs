use serde::{Deserialize, Serialize};

/// Открытый экран — одиннадцать экранов §13.3 плюс заставка.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Screen {
    Boot,
    NewLife,
    Market,
    Event,
    Travel,
    Businesses,
    Manager,
    Contacts,
    Money,
    Pager,
    Notebook,
    Zone,
    Summary,
}
