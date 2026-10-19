use serde::{Deserialize, Serialize};

use crate::economy::Real;

/// Учёт текущего месяца дела: сколько дней оно было, сколько из них с управляющим, сколько
/// работало и сколько ушло на откупы.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonthLedger {
    pub days: u32,
    pub staffed_days: u32,
    pub working_days: u32,
    pub bribes: Real,
}
