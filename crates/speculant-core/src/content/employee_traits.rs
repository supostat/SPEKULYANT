use serde::Deserialize;

use super::HabitId;

/// Черты человека, которого можно нанять управляющим (§7.2): навык (хватка), честность и
/// лояльность — деления шкалы правил дел — и привычки.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeTraits {
    pub skill: u8,
    pub honesty: u8,
    pub loyalty: u8,
    #[serde(default)]
    pub habits: Vec<HabitId>,
}
