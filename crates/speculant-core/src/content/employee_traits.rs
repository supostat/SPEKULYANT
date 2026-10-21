use serde::Deserialize;

use super::HabitId;
use crate::time::GameDate;

/// Черты человека, которого можно нанять управляющим (§7.2): навык (хватка), честность и
/// лояльность — деления шкалы правил дел, — привычки и день, с которого он ищет работу.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmployeeTraits {
    pub skill: u8,
    pub honesty: u8,
    pub loyalty: u8,
    #[serde(default)]
    pub habits: Vec<HabitId>,
    /// С какого дня человек ищет работу: афганец — после вывода войск, инженер закрытого НИИ —
    /// когда перестали платить; без поля — с начала игры.
    pub from: Option<GameDate>,
}

impl EmployeeTraits {
    /// День, с которого человек начнёт искать работу, если он ещё не наступил; `None` — уже ищет.
    pub fn waits_until(&self, today: GameDate) -> Option<GameDate> {
        self.from.filter(|from| today < *from)
    }
}
