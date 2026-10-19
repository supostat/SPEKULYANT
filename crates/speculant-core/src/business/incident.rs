use serde::Deserialize;

/// Происшествие с делом (§7.2, §7.3). Одни политика закрывает сама — крыша разбирается с рэкетом,
/// управляющий откупается от проверки или закрывается; остальные выходят за политики и приходят
/// событием `data/events/<эпоха>/` с полем `incident`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Incident {
    /// Рэкет требует долю; с крышей — её забота.
    Racket,
    /// Проверка; что делать — решает политика проверки.
    Inspection,
    /// Крыша поднимает ставку — всегда вопрос к хозяину.
    RoofRaise,
    /// Управляющий уходит — к конкурентам или просто так.
    Departure,
    /// Управляющий сдаёт хозяина.
    Betrayal,
    /// Пожар.
    Fire,
}

impl Incident {
    /// Порядок, в котором дело разыгрывает происшествия дня, — он часть детерминизма.
    pub const ALL: [Self; 6] = [
        Self::Racket,
        Self::Inspection,
        Self::RoofRaise,
        Self::Departure,
        Self::Betrayal,
        Self::Fire,
    ];
}
