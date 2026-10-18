use serde::{Deserialize, Serialize};

use super::{GameDate, TimeRulesError};

pub const MAX_SPEED: u8 = 3;

/// Длительность игрового дня в миллисекундах реального времени на каждой скорости и при перемотке.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClockSettings {
    day_milliseconds: [u32; MAX_SPEED as usize],
    fast_forward_day_milliseconds: u32,
}

impl ClockSettings {
    /// Скорости строго ускоряются, а перемотка быстрее самой быстрой скорости.
    pub fn new(
        day_milliseconds: [u32; MAX_SPEED as usize],
        fast_forward_day_milliseconds: u32,
    ) -> Result<Self, TimeRulesError> {
        let speeds_accelerate =
            day_milliseconds[0] > 0 && day_milliseconds.windows(2).all(|pair| pair[1] < pair[0]);
        if !speeds_accelerate {
            return Err(TimeRulesError::SpeedsNotAccelerating);
        }
        let fastest = day_milliseconds[day_milliseconds.len() - 1];
        if fast_forward_day_milliseconds == 0 || fast_forward_day_milliseconds >= fastest {
            return Err(TimeRulesError::FastForwardNotFastest);
        }
        Ok(Self {
            day_milliseconds,
            fast_forward_day_milliseconds,
        })
    }

    fn day_milliseconds(&self, speed: u8, fast_forward: bool) -> u64 {
        let milliseconds = if fast_forward {
            self.fast_forward_day_milliseconds
        } else {
            self.day_milliseconds[usize::from(speed) - 1]
        };
        u64::from(milliseconds)
    }
}

/// Перемотка спокойного периода: до ближайшего события или до прибытия из поездки.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FastForward {
    UntilEvent,
    UntilArrival,
}

/// Наступивший день.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DayTick {
    pub date: GameDate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvalidSpeed {
    pub speed: u8,
}

/// Часы мира: превращают реальное время в игровые дни по скорости, стоят на паузе, перематывают.
#[derive(Clone, Debug)]
pub struct Clock {
    settings: ClockSettings,
    today: GameDate,
    speed: u8,
    paused: bool,
    fast_forward: Option<FastForward>,
    carried_milliseconds: u64,
}

impl Clock {
    pub fn new(settings: ClockSettings, today: GameDate) -> Self {
        Self {
            settings,
            today,
            speed: 1,
            paused: false,
            fast_forward: None,
            carried_milliseconds: 0,
        }
    }

    pub fn today(&self) -> GameDate {
        self.today
    }

    pub fn speed(&self) -> u8 {
        self.speed
    }

    pub fn is_paused(&self) -> bool {
        self.paused
    }

    pub fn fast_forward(&self) -> Option<FastForward> {
        self.fast_forward
    }

    /// Новая скорость 1–3; игрок взял время в руки, поэтому перемотка прекращается.
    pub fn set_speed(&mut self, speed: u8) -> Result<(), InvalidSpeed> {
        if !(1..=MAX_SPEED).contains(&speed) {
            return Err(InvalidSpeed { speed });
        }
        self.speed = speed;
        self.fast_forward = None;
        Ok(())
    }

    /// Пауза останавливает и ход времени, и перемотку.
    pub fn set_paused(&mut self, paused: bool) {
        self.paused = paused;
        if paused {
            self.fast_forward = None;
        }
    }

    pub fn start_fast_forward(&mut self, until: FastForward) {
        self.paused = false;
        self.fast_forward = Some(until);
    }

    /// Мир достиг цели перемотки — пришло событие или поездка закончилась.
    pub fn stop_fast_forward(&mut self) {
        self.fast_forward = None;
    }

    /// Двигает часы на `milliseconds` реального времени и отдаёт наступившие дни разом.
    pub fn advance(&mut self, milliseconds: u32) -> Vec<DayTick> {
        self.add_time(milliseconds);
        std::iter::from_fn(|| self.next_day()).collect()
    }

    /// Копит реальное время; на паузе время не идёт.
    pub fn add_time(&mut self, milliseconds: u32) {
        if !self.paused {
            self.carried_milliseconds += u64::from(milliseconds);
        }
    }

    /// Следующий наступивший день, если накопленного времени на него хватает; недобранный до
    /// целого дня остаток копится дальше.
    pub fn next_day(&mut self) -> Option<DayTick> {
        let day_length = self
            .settings
            .day_milliseconds(self.speed, self.fast_forward.is_some());
        if self.paused || self.carried_milliseconds < day_length {
            return None;
        }
        self.carried_milliseconds -= day_length;
        self.today = self.today.next();
        Some(DayTick { date: self.today })
    }

    /// Мир остановился на событии: накопленное время сгорает, дни дальше не идут.
    pub fn drop_carried_time(&mut self) {
        self.carried_milliseconds = 0;
    }
}
