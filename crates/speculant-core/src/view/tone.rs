use serde::{Deserialize, Serialize};

/// Тон пищалки PC-speaker: частота в целых герцах — без неё пауза — и длительность.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToneView {
    pub hertz: Option<u32>,
    pub milliseconds: u32,
}
