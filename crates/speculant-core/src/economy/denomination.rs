use serde::Deserialize;

use crate::time::GameDate;

/// Деноминация (§3.3): с этой даты рубль на ценнике в `factor` раз крупнее прежнего — «три нуля».
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Denomination {
    pub date: GameDate,
    pub factor: i64,
}

impl Denomination {
    /// Во сколько раз рубль на ценнике этого дня крупнее рубля до деноминации.
    pub fn divisor_on(self, date: GameDate) -> i64 {
        if date >= self.date { self.factor } else { 1 }
    }
}
