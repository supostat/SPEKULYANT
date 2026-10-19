use serde::Deserialize;

use super::{CityId, FlagId, HousingId, NpcId};
use crate::economy::Real;

/// Стартовая биография (§11.1, решение 15.5): где, с чем и в каком возрасте начинается жизнь,
/// первая фраза некролога и, если биография закрыта, чем её открыть.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Biography {
    pub title: String,
    pub description: String,
    pub start_city: CityId,
    /// Наличные на старте в рублях 1984 года.
    pub start_cash: Real,
    /// Знакомые на старте — связи (§9).
    #[serde(default)]
    pub contacts: Vec<NpcId>,
    /// Сколько лет персонажу 1 января 1984 года: сид выбирает от и до.
    pub age: AgeRange,
    pub housing: HousingId,
    /// С чего начинается некролог: «Начинал фарцовщиком у гостиницы «Интурист».».
    pub obituary: String,
    pub starts_unlocked: bool,
    /// Флаг, поставленный в жизни, открывает биографию для следующих.
    pub unlocked_by_flag: Option<FlagId>,
    /// Подсказка экрана выбора для закрытой биографии.
    pub unlock_hint: Option<String>,
}

/// Возраст от и до включительно.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgeRange {
    pub from: u32,
    pub to: u32,
}
