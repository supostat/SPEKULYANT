use crate::economy::Real;
use crate::time::GameDate;

/// Как политика обошлась с происшествием.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Handling {
    /// Крыша разобралась с рэкетом.
    ByRoof,
    /// Управляющий откупился от проверки — сумма уйдёт из выручки месяца.
    Bribed(Real),
    /// Управляющий закрыл дело до даты.
    Closed { until: GameDate },
    /// Политика этого не предусматривает — решать хозяину.
    OutOfPolicy,
}
