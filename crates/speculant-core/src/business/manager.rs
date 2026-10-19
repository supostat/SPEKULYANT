use super::Employee;
use crate::time::GameDate;

/// Управляющий делом (§7.3): нанятый человек и день найма. Политики принадлежат делу и
/// переходят к следующему управляющему.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Manager {
    employee: Employee,
    since: GameDate,
}

impl Manager {
    pub fn new(employee: Employee, since: GameDate) -> Self {
        Self { employee, since }
    }

    pub fn employee(&self) -> &Employee {
        &self.employee
    }

    pub fn employee_mut(&mut self) -> &mut Employee {
        &mut self.employee
    }

    pub fn since(&self) -> GameDate {
        self.since
    }
}
