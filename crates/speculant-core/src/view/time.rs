use serde::{Deserialize, Serialize};

use crate::time::FastForward;

/// Ход времени: скорость 1–3, пауза (пробел) и перемотка — до события или до прибытия.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeView {
    pub speed: u8,
    pub paused: bool,
    pub fast_forward: Option<FastForward>,
}
