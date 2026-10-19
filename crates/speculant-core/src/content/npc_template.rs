use serde::Deserialize;

use super::FactionId;
use crate::economy::Real;
use crate::npc::Strategy;

/// Шаблон соперника или знакомого (§8, §9): из пула шаблонов сид выбирает состав мира.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcTemplate {
    pub name: String,
    pub role: String,
    /// Фракция, к которой человек принадлежит: участковый — милиция, фарцовщик — торговцы.
    pub faction: FactionId,
    /// Как зарабатывает соперник; у знакомого стратегии нет — он не соперник.
    pub strategy: Option<Strategy>,
    /// Стартовый капитал соперника в рублях 1984 года; есть только у соперников.
    pub capital: Option<Real>,
    pub description: String,
}
