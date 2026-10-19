use serde::Deserialize;

/// Что делает управляющий, когда приходит проверка (§7.3).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InspectionResponse {
    /// Откупается из кассы — в рамках политики.
    Pay,
    /// Звонит хозяину — проверка выходит за политику и приходит событием.
    Call,
    /// Закрывает дело на несколько дней — в рамках политики.
    Close,
}
