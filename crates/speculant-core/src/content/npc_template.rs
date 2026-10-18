use serde::Deserialize;

use super::FactionId;

/// Шаблон соперника или знакомого (§8, §9): из пула шаблонов сид выбирает состав мира.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcTemplate {
    pub name: String,
    pub role: String,
    /// Фракция, к которой человек принадлежит: участковый — милиция, фарцовщик — торговцы.
    pub faction: FactionId,
    /// Как торгует соперник; у знакомого, который не торгует, стратегии нет.
    pub strategy: Option<String>,
    pub description: String,
}
