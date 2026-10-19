use serde::Deserialize;

use crate::time::GameDate;

/// Форма собственности (§7.1) из `data/business/forms/<форма>.toml`: подпольный цех, кооператив,
/// ТОО, ООО, банк — с окном дат, когда в ней можно работать.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BusinessForm {
    pub title: String,
    pub from: GameDate,
    /// С этой даты форма закрыта — дела переходят в следующую.
    pub until: Option<GameDate>,
    /// Сдвиг уровня наезда: подпольный цех заметнее.
    #[serde(default)]
    pub raid: i32,
    /// Доля наездов от рэкета; остальные — проверки.
    pub racket_share: f64,
    /// Налог — доля выручки.
    #[serde(default)]
    pub tax: f64,
}

impl BusinessForm {
    pub fn is_open_on(&self, date: GameDate) -> bool {
        self.from <= date && self.until.is_none_or(|until| date < until)
    }
}
