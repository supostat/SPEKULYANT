use serde::Deserialize;

/// Шаблон соперника или знакомого (§8): из пула шаблонов сид выбирает состав мира.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NpcTemplate {
    pub name: String,
    pub role: String,
    pub strategy: String,
    pub description: String,
}
