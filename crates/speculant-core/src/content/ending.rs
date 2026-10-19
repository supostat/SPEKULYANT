use std::collections::BTreeMap;

use serde::Deserialize;

use super::EpochId;
use crate::life::EndingKind;
use crate::time::GameDate;

/// Финал жизни (§11.2) из `data/endings/<финал>.toml`: каким он бывает, выбирает ли его игрок сам
/// и заметка в духе эпохи — чья она, заголовок и последняя фраза. Подстановки: `{name}`, `{NAME}`
/// (прописными), `{born}` и `{ended}` (годы), `{date}` («3 ноября 1997»), `{age}`.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ending {
    pub kind: EndingKind,
    /// Как финал зовётся в меню ухода.
    pub title: String,
    /// Игрок выбирает его сам: эмиграция, покой.
    #[serde(default)]
    pub voluntary: bool,
    /// С этой даты добровольный финал можно выбрать.
    pub available_from: Option<GameDate>,
    /// «Капитал на момент смерти».
    pub capital_label: String,
    /// Чья заметка в каждую эпоху: газета некролога, суд приговора.
    pub sources: BTreeMap<EpochId, String>,
    pub headline: String,
    pub text: String,
}

impl Ending {
    /// Добровольный финал можно выбрать в этот день.
    pub fn is_open_on(&self, date: GameDate) -> bool {
        self.voluntary && self.available_from.is_none_or(|from| from <= date)
    }
}
