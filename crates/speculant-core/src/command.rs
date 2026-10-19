//! Команды игрока — единственный путь, которым интерфейс меняет мир.

use serde::{Deserialize, Serialize};

use crate::time::FastForward;
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
    /// Перемотать спокойное время: до ближайшего события или до прибытия из поездки.
    FastForward {
        until: FastForward,
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
    /// Открыть точку дела этого типа в городе, где игрок сейчас: первая открывает дело, следующие
    /// расширяют его.
    OpenBusiness {
        kind: String,
    },
    /// Вариант политики дела — номер от нуля, как `selected` в снимке.
    SetPolicy {
        business: String,
        policy: String,
        option: usize,
    },
    /// Нанять управляющим человека из кандидатов снимка.
    Hire {
        business: String,
        candidate: String,
    },
    /// Уволить управляющего дела.
    Fire {
        business: String,
        employee: String,
    },
    Call {
        contact: String,
    },
    /// Купить жильё за наличные; прежнее уходит в зачёт.
    BuyHousing {
        housing: String,
    },
    /// Уйти по своей воле — эмиграция или покой: жизнь кончается итогами.
    Retire {
        ending: String,
    },
    /// В зоне: ходить на промзону или нет.
    SetWork {
        work: bool,
    },
    /// В зоне: услуга за сигареты и чай.
    UseService {
        service: String,
    },
    /// В зоне: передача с воли за наличные.
    OrderParcel,
    /// В зоне: просьба об УДО.
    ApplyForParole,
    /// В зоне: выход за взятку наличными.
    BribeForRelease,
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
            Self::FastForward { .. } => "fast_forward",
            Self::Buy { .. } => "buy",
            Self::Sell { .. } => "sell",
            Self::Travel { .. } => "travel",
            Self::Choose { .. } => "choose",
            Self::Continue => "continue",
            Self::OpenScreen { .. } => "open_screen",
            Self::OpenBusiness { .. } => "open_business",
            Self::SetPolicy { .. } => "set_policy",
            Self::Hire { .. } => "hire",
            Self::Fire { .. } => "fire",
            Self::Call { .. } => "call",
            Self::BuyHousing { .. } => "buy_housing",
            Self::Retire { .. } => "retire",
            Self::SetWork { .. } => "set_work",
            Self::UseService { .. } => "use_service",
            Self::OrderParcel => "order_parcel",
            Self::ApplyForParole => "apply_for_parole",
            Self::BribeForRelease => "bribe_for_release",
            Self::Deposit { .. } => "deposit",
            Self::Withdraw { .. } => "withdraw",
            Self::Save { .. } => "save",
            Self::Load { .. } => "load",
            Self::Quit => "quit",
        }
    }
}
