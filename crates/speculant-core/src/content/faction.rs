use serde::Deserialize;

use super::EpochId;
use crate::time::{EpochTable, GameDate};

/// Фракция репутации (§9, решение 15.6) из `data/factions/<фракция>.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Faction {
    pub title: String,
    /// Эпоха, с которой фракция есть в мире (силовики — со стабилизации); без неё — с начала игры.
    pub from_epoch: Option<EpochId>,
}

impl Faction {
    pub fn is_visible_on(&self, date: GameDate, epochs: &EpochTable) -> bool {
        self.from_epoch.as_ref().is_none_or(|epoch| {
            epochs
                .epoch(epoch.as_str())
                .is_some_and(|epoch| epoch.starts <= date)
        })
    }
}
