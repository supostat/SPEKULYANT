use serde::{Deserialize, Serialize};

use crate::economy::Money;
use crate::time::GameDate;

/// Отчёт управляющего за месяц: выручка и всё, что из неё ушло, итог, который пришёл хозяину или
/// который хозяин доплатил, и что стало с хваткой управляющего. Суммы — рубли дня расчёта.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MonthReport {
    /// Первое число месяца, в которое считали прошлый месяц.
    pub settled_on: GameDate,
    pub income: Money,
    pub roof: Money,
    pub tax: Money,
    pub rent: Money,
    pub wages: Money,
    pub stolen: Money,
    pub bribes: Money,
    /// Выручка минус все расходы; меньше нуля — дело в убытке.
    pub profit: Money,
    /// За месяц управляющий пропил деление хватки.
    pub skill_lost: bool,
}
