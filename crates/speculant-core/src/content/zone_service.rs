use serde::Deserialize;

use crate::zone::Stash;

/// Услуга зоны (§12) из `data/zone/services/<услуга>.toml`: цена в сигаретах и чае и что она даёт
/// — здоровье (больничка) и авторитет (общак).
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneService {
    pub title: String,
    pub price: Stash,
    #[serde(default)]
    pub health: i32,
    #[serde(default)]
    pub authority: i32,
}
