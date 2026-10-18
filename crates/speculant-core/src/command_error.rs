//! Отказ ядра выполнить команду — с причиной, которую интерфейс может показать игроку.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum CommandError {
    EmptyBiography,
    UnknownBiography {
        biography: String,
    },
    InvalidSpeed {
        speed: u8,
        max: u8,
    },
    UnknownGood {
        good: String,
    },
    NotOnSale {
        good: String,
        name: String,
    },
    ZeroQuantity,
    OutOfStock {
        good: String,
        name: String,
        available: u32,
    },
    NotEnoughCash {
        needed: String,
        cash: String,
    },
    NotInPocket {
        good: String,
        name: String,
        held: u32,
    },
    UnknownPlace {
        place: String,
    },
    ZeroAmount,
    PlaceClosed {
        place: String,
        title: String,
    },
    NotEnoughInPlace {
        place: String,
        title: String,
        balance: String,
    },
    UnknownEvent {
        event: String,
    },
    EventNotReady {
        event: String,
    },
    EventPending,
    NoPendingEvent,
    NoSuchChoice {
        choice: usize,
        count: usize,
    },
    ChoiceUnavailable {
        choice: usize,
    },
    ChoiceAlreadyMade,
    ChoiceRequired,
    NotSimulatedYet {
        command: String,
    },
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBiography => write!(formatter, "жизнь не начинается без биографии"),
            Self::UnknownBiography { biography } => {
                write!(formatter, "биографии {biography} нет")
            }
            Self::InvalidSpeed { speed, max } => {
                write!(formatter, "скорости {speed} нет, есть 1–{max}")
            }
            Self::UnknownGood { good } => write!(formatter, "товара {good} нет"),
            Self::NotOnSale { name, .. } => write!(formatter, "{name}: такого ещё не продают"),
            Self::ZeroQuantity => write!(formatter, "количество должно быть больше нуля"),
            Self::OutOfStock {
                name, available, ..
            } => write!(formatter, "{name}: на рынке только {available}"),
            Self::NotEnoughCash { needed, cash } => {
                write!(
                    formatter,
                    "не хватает денег: нужно {needed}, при себе {cash}"
                )
            }
            Self::NotInPocket { name, held, .. } => {
                write!(formatter, "{name}: в кармане только {held}")
            }
            Self::UnknownPlace { place } => write!(formatter, "места хранения {place} нет"),
            Self::ZeroAmount => write!(formatter, "сумма должна быть больше нуля"),
            Self::PlaceClosed { title, .. } => {
                write!(formatter, "{title}: сейчас туда не положить")
            }
            Self::NotEnoughInPlace { title, balance, .. } => {
                write!(formatter, "{title}: там только {balance}")
            }
            Self::UnknownEvent { event } => write!(formatter, "события {event} нет"),
            Self::EventNotReady { event } => {
                write!(formatter, "событие {event} сейчас случиться не может")
            }
            Self::EventPending => write!(formatter, "время стоит: сначала ответь на событие"),
            Self::NoPendingEvent => write!(formatter, "сейчас ничего не происходит"),
            Self::NoSuchChoice { choice, count } => {
                write!(formatter, "выбора {choice} нет, есть 1–{count}")
            }
            Self::ChoiceUnavailable { choice } => {
                write!(formatter, "выбор {choice} сейчас недоступен")
            }
            Self::ChoiceAlreadyMade => write!(formatter, "выбор уже сделан"),
            Self::ChoiceRequired => write!(formatter, "сначала выбери, что делать"),
            Self::NotSimulatedYet { command } => {
                write!(formatter, "команда {command} ещё не симулируется ядром")
            }
        }
    }
}

impl std::error::Error for CommandError {}
