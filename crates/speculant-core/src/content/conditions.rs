use serde::Deserialize;

use super::{CityId, EpochId, FactionId, FlagId, GoodId, NpcId, TimelineNodeId};
use crate::economy::Real;
use crate::time::GameDate;

/// Условия события или выбора (§10.1) — структурные таблицы, а не строки-выражения.
/// Пустые условия выполняются всегда.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Conditions {
    #[serde(default)]
    pub epochs: Vec<EpochId>,
    pub window: Option<DateWindow>,
    #[serde(default)]
    pub cities: Vec<CityId>,
    #[serde(default)]
    pub reputation: Vec<ReputationCondition>,
    pub money: Option<MoneyCondition>,
    #[serde(default)]
    pub goods: Vec<GoodCondition>,
    #[serde(default)]
    pub flags_set: Vec<FlagId>,
    #[serde(default)]
    pub flags_unset: Vec<FlagId>,
    pub npc: Option<NpcId>,
    pub timeline: Option<TimelineNodeId>,
}

/// Окно дат; без `until` окно открыто до конца жизни.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DateWindow {
    pub from: GameDate,
    pub until: Option<GameDate>,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReputationCondition {
    pub faction: FactionId,
    pub at_least: Option<i32>,
    pub at_most: Option<i32>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoneyCondition {
    pub at_least: Real,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoodCondition {
    pub good: GoodId,
    pub at_least: u32,
}
