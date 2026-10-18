use serde::Deserialize;

use super::{CityId, FactionId, FlagId, GoodId, NpcId};
use crate::economy::Real;

/// Последствия выбора (§10.1): деньги, товар, репутация, связи, флаги, сдвиг цен.
#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Consequences {
    pub money: Option<Real>,
    #[serde(default)]
    pub goods: Vec<GoodChange>,
    #[serde(default)]
    pub reputation: Vec<ReputationChange>,
    #[serde(default)]
    pub contacts: Vec<ContactChange>,
    #[serde(default)]
    pub set_flags: Vec<FlagId>,
    #[serde(default)]
    pub clear_flags: Vec<FlagId>,
    #[serde(default)]
    pub price_shocks: Vec<PriceShock>,
}

impl Consequences {
    pub fn is_empty(&self) -> bool {
        self == &Self::default()
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoodChange {
    pub good: GoodId,
    pub change: i32,
}

#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReputationChange {
    pub faction: FactionId,
    pub change: i32,
}

/// Перемена в связи с конкретным человеком (§9): отношение (плюс — расположение, минус — обида),
/// услуги (плюс — он должен игроку, минус — игрок ему), отъезд на `away_days` дней или потеря.
/// Незнакомый человек после такого последствия становится связью.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContactChange {
    pub npc: NpcId,
    #[serde(default)]
    pub relation: i32,
    #[serde(default)]
    pub favours: i32,
    pub away_days: Option<u32>,
    #[serde(default)]
    pub lost: bool,
}

impl ContactChange {
    /// Последствие что-то меняет.
    pub fn changes_anything(&self) -> bool {
        self.relation != 0 || self.favours != 0 || self.away_days.is_some() || self.lost
    }
}

/// Шок цены (§5.2 п. 4): множитель кривой товара — в одном городе или везде, на срок или навсегда.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriceShock {
    pub good: GoodId,
    pub city: Option<CityId>,
    pub factor: f64,
    pub days: Option<u32>,
}
