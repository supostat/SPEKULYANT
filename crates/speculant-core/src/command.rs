//! Команды игрока — единственный путь, которым интерфейс меняет мир.

use serde::{Deserialize, Serialize};

use crate::view::Screen;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "command", rename_all = "snake_case")]
pub enum Command {
    NewLife(LifeSettings),
    Advance {
        milliseconds: u32,
    },
    SetSpeed {
        speed: u8,
    },
    Pause {
        paused: bool,
    },
    Buy {
        good: String,
        quantity: u32,
    },
    Sell {
        good: String,
        quantity: u32,
    },
    Travel {
        destination: String,
    },
    /// Выбор в событии, остановившем время: номер от 1, как на экране.
    Choose {
        choice: usize,
    },
    /// Исход события прочитан — время идёт дальше.
    Continue,
    OpenScreen {
        screen: Screen,
    },
    SetPolicy {
        business: String,
        policy: String,
        option: usize,
    },
    Hire {
        business: String,
        candidate: String,
    },
    Fire {
        business: String,
        employee: String,
    },
    Call {
        contact: String,
    },
    /// Положить в место хранения `amount` целых единиц его валюты на ценнике дня: рубли — из
    /// наличных, чеки и доллары — купленные на наличные по курсу.
    Deposit {
        place: String,
        amount: u64,
    },
    /// Забрать из места хранения `amount` целых единиц его валюты в наличные рубли по курсу дня.
    Withdraw {
        place: String,
        amount: u64,
    },
    Save {
        slot: String,
    },
    Load {
        slot: String,
    },
    Quit,
}

/// Старт жизни: биография, режим сохранений и сид истории.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeSettings {
    pub biography: String,
    pub ironman: bool,
    pub seed: u64,
}

impl Command {
    /// Имя команды так, как оно стоит в поле `command` JSON.
    pub fn name(&self) -> &'static str {
        match self {
            Self::NewLife(_) => "new_life",
            Self::Advance { .. } => "advance",
            Self::SetSpeed { .. } => "set_speed",
            Self::Pause { .. } => "pause",
            Self::Buy { .. } => "buy",
            Self::Sell { .. } => "sell",
            Self::Travel { .. } => "travel",
            Self::Choose { .. } => "choose",
            Self::Continue => "continue",
            Self::OpenScreen { .. } => "open_screen",
            Self::SetPolicy { .. } => "set_policy",
            Self::Hire { .. } => "hire",
            Self::Fire { .. } => "fire",
            Self::Call { .. } => "call",
            Self::Deposit { .. } => "deposit",
            Self::Withdraw { .. } => "withdraw",
            Self::Save { .. } => "save",
            Self::Load { .. } => "load",
            Self::Quit => "quit",
        }
    }
}
