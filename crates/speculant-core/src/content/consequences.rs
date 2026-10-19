use serde::Deserialize;

use super::{CityId, EndingId, FactionId, FlagId, GoodId, NpcId};
use crate::business::PolicyKind;
use crate::economy::Real;
use crate::zone::Stash;

/// Последствия выбора (§10.1): деньги, товар, репутация, связи, флаги, сдвиг цен, перемены в деле,
/// здоровье, срок и перемены в зоне, конец жизни.
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
    /// Перемена в отношениях с соперником, который участвует в событии.
    pub rival: Option<RivalChange>,
    #[serde(default)]
    pub set_flags: Vec<FlagId>,
    #[serde(default)]
    pub clear_flags: Vec<FlagId>,
    #[serde(default)]
    pub price_shocks: Vec<PriceShock>,
    /// Перемена в деле, с которым случилось происшествие события.
    pub business: Option<BusinessChange>,
    /// Здоровье: пьянство, драка, санаторий; на нуле — финал здоровья.
    #[serde(default)]
    pub health: i32,
    /// Суд дал срок в месяцах — игрок идёт в зону (§12), а если уже сидит, срок прибавляется;
    /// срок длиннее порога правил зоны кончает жизнь приговором. Только у событий на воле.
    pub sentence_months: Option<u32>,
    /// Перемена в зоне — только у событий зоны.
    pub zone: Option<ZoneChange>,
    /// Жизнь кончается этим финалом, когда игрок прочтёт исход.
    pub end_life: Option<EndingId>,
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

/// Перемена в отношениях с участвующим соперником: отношение и услуги, как у связи.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RivalChange {
    #[serde(default)]
    pub relation: i32,
    #[serde(default)]
    pub favours: i32,
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

/// Перемена в деле происшествия (§7.3): новые варианты политик, уход управляющего, закрытие на
/// `closed_days` дней, потеря дела.
#[derive(Clone, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BusinessChange {
    #[serde(default)]
    pub policies: Vec<PolicySetting>,
    #[serde(default)]
    pub manager_leaves: bool,
    pub closed_days: Option<u32>,
    #[serde(default)]
    pub lost: bool,
}

/// Вариант политики по подписи: «крыше — да, 20%».
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicySetting {
    pub policy: PolicyKind,
    pub option: String,
}

impl BusinessChange {
    /// Последствие что-то меняет.
    pub fn changes_anything(&self) -> bool {
        !self.policies.is_empty() || self.manager_leaves || self.closed_days.is_some() || self.lost
    }
}

/// Перемена в зоне (§12): сигареты и чай (минус — отдать, и их должно хватать), авторитет, месяцы
/// к сроку или из срока и выход по амнистии.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct ZoneChange {
    pub cigarettes: i32,
    pub tea: i32,
    pub authority: i32,
    pub term_months: i32,
    pub amnesty: bool,
}

impl ZoneChange {
    /// Последствие что-то меняет.
    pub fn changes_anything(&self) -> bool {
        self != &Self::default()
    }

    /// Сигареты и чай, которые выбор забирает.
    pub fn price(&self) -> Stash {
        Stash {
            cigarettes: self.cigarettes.min(0).unsigned_abs(),
            tea: self.tea.min(0).unsigned_abs(),
        }
    }

    /// Сигареты и чай, которые выбор приносит.
    pub fn gain(&self) -> Stash {
        Stash {
            cigarettes: self.cigarettes.max(0).unsigned_abs(),
            tea: self.tea.max(0).unsigned_abs(),
        }
    }
}
