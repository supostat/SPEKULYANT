//! Фасад ядра: интерфейс и симулятор видят мир только через `Game`.

use crate::command::{Command, LifeSettings};
use crate::command_error::CommandError;
use crate::view::StateView;

const MAX_SPEED: u8 = 3;
const OPENING_SNAPSHOT: &str = include_str!("../../../design/etalons/market.view.json");

#[derive(Debug)]
pub struct Game {
    view: StateView,
}

impl Game {
    /// Открывает жизнь. Мир пока не симулируется, поэтому жизнь начинается со снимка рынка
    /// дизайн-книги (`design/etalons/market.view.json`); фаза core-time заменит снимок миром по сиду.
    pub fn new_life(settings: &LifeSettings) -> Result<Self, CommandError> {
        if settings.biography.trim().is_empty() {
            return Err(CommandError::EmptyBiography);
        }
        let view = serde_json::from_str(OPENING_SNAPSHOT)
            .expect("снимок рынка дизайн-книги соответствует StateView");
        Ok(Self { view })
    }

    pub fn view(&self) -> &StateView {
        &self.view
    }

    /// Двигает мир на `milliseconds` реального времени и сообщает, изменился ли снимок.
    /// Календаря ещё нет — он появится в фазе core-time, — поэтому мир стоит.
    pub fn advance(&mut self, _milliseconds: u32) -> bool {
        false
    }

    pub fn apply(&mut self, command: Command) -> Result<(), CommandError> {
        match command {
            Command::NewLife(settings) => {
                *self = Self::new_life(&settings)?;
                Ok(())
            }
            Command::Advance { milliseconds } => {
                self.advance(milliseconds);
                Ok(())
            }
            Command::SetSpeed { speed } => {
                if !(1..=MAX_SPEED).contains(&speed) {
                    return Err(CommandError::InvalidSpeed {
                        speed,
                        max: MAX_SPEED,
                    });
                }
                self.view.time.speed = speed;
                Ok(())
            }
            Command::Pause { paused } => {
                self.view.time.paused = paused;
                Ok(())
            }
            Command::OpenScreen { screen } => {
                self.view.screen = screen;
                Ok(())
            }
            other => Err(CommandError::NotSimulatedYet {
                command: other.name().to_owned(),
            }),
        }
    }
}
