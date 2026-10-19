use serde::Deserialize;

use super::{EmployeeTraits, FactionId};
use crate::economy::Real;
use crate::npc::Strategy;

/// Шаблон соперника, знакомого или того, кого можно нанять (§7.2, §8, §9): из пула шаблонов сид
/// выбирает состав соперников мира.
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
    /// Черты, если человека можно нанять управляющим; соперник не нанимается.
    pub employee: Option<EmployeeTraits>,
    pub description: String,
}
