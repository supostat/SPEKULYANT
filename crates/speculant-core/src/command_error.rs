//! Отказ ядра выполнить команду — с причиной, которую интерфейс может показать игроку.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum CommandError {
    EmptyBiography,
    InvalidSpeed { speed: u8, max: u8 },
    NotSimulatedYet { command: String },
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBiography => write!(formatter, "жизнь не начинается без биографии"),
            Self::InvalidSpeed { speed, max } => {
                write!(formatter, "скорости {speed} нет, есть 1–{max}")
            }
            Self::NotSimulatedYet { command } => {
                write!(formatter, "команда {command} ещё не симулируется ядром")
            }
        }
    }
}

impl std::error::Error for CommandError {}
