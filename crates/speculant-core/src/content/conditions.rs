use serde::Deserialize;

use super::{CityId, EpochId, FactionId, FlagId, GoodId, NpcId, TimelineNodeId};
use crate::economy::Real;
use crate::npc::Strategy;
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
    /// Участвует соперник, подходящий под условие; кто именно — решает сид при начале события.
    pub rival: Option<RivalCondition>,
    pub timeline: Option<TimelineNodeId>,
    /// Уровень авторитета в зоне — только у событий зоны.
    pub authority: Option<AuthorityCondition>,
}

/// Границы авторитета в зоне, уровни от 1.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AuthorityCondition {
    pub at_least: Option<u8>,
    pub at_most: Option<u8>,
}

/// Какой соперник участвует (§8): стратегия, отношение к игроку и давность обиды — так «кинутый
/// партнёр возвращается через полгода».
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RivalCondition {
    pub strategy: Option<Strategy>,
    pub relation_at_least: Option<i32>,
    pub relation_at_most: Option<i32>,
    pub offended_days: Option<u32>,
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
