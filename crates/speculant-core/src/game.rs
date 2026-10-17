//! Фасад ядра: интерфейс и симулятор видят мир только через `Game`.

use crate::command::{Command, LifeSettings};
use crate::command_error::CommandError;
use crate::time::{Clock, GameDate, MAX_SPEED, TimeRules};
use crate::view::{StateView, TimeView};

const OPENING_SNAPSHOT: &str = include_str!("../../../design/etalons/market.view.json");

#[derive(Debug)]
pub struct Game {
    rules: TimeRules,
    clock: Clock,
    view: StateView,
}

impl Game {
    /// Открывает жизнь в первый день 1984 года. Дата, скорость и пауза — настоящие, от часов мира.
    /// Рынок, карман, слухи и сообщения пока берутся из снимка рынка дизайн-книги
    /// (`design/etalons/market.view.json`) — их заменят экономика и контент мира.
    pub fn new_life(settings: &LifeSettings, rules: &TimeRules) -> Result<Self, CommandError> {
        if settings.biography.trim().is_empty() {
            return Err(CommandError::EmptyBiography);
        }
        let view = serde_json::from_str(OPENING_SNAPSHOT)
            .expect("снимок рынка дизайн-книги соответствует StateView");
        let mut game = Self {
            rules: rules.clone(),
            clock: Clock::new(rules.clock.clone(), GameDate::START),
            view,
        };
        game.refresh_time();
        Ok(game)
    }

    pub fn view(&self) -> &StateView {
        &self.view
    }

    pub fn today(&self) -> GameDate {
        self.clock.today()
    }

    /// Двигает мир на `milliseconds` реального времени; `true`, если наступил хотя бы один день.
    pub fn advance(&mut self, milliseconds: u32) -> bool {
        let days_passed = !self.clock.advance(milliseconds).is_empty();
        if days_passed {
            self.refresh_time();
        }
        days_passed
    }

    pub fn apply(&mut self, command: Command) -> Result<(), CommandError> {
        match command {
            Command::NewLife(settings) => {
                *self = Self::new_life(&settings, &self.rules)?;
            }
            Command::Advance { milliseconds } => {
                self.advance(milliseconds);
            }
            Command::SetSpeed { speed } => {
                self.clock
                    .set_speed(speed)
                    .map_err(|invalid| CommandError::InvalidSpeed {
                        speed: invalid.speed,
                        max: MAX_SPEED,
                    })?;
                self.refresh_time();
            }
            Command::Pause { paused } => {
                self.clock.set_paused(paused);
                self.refresh_time();
            }
            Command::OpenScreen { screen } => {
                self.view.screen = screen;
            }
            other => {
                return Err(CommandError::NotSimulatedYet {
                    command: other.name().to_owned(),
                });
            }
        }
        Ok(())
    }

    fn refresh_time(&mut self) {
        self.view.header.date = self.clock.today().russian();
        self.view.time = TimeView {
            speed: self.clock.speed(),
            paused: self.clock.is_paused(),
        };
    }
}
