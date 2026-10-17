use serde::{Deserialize, Serialize};

/// Ход времени: скорость 1–3 и пауза (пробел).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeView {
    pub speed: u8,
    pub paused: bool,
}
