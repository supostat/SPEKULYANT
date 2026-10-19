use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::ReputationRules;
use crate::content::{Faction, FactionId};

/// Репутация по фракциям (§9): целый уровень у каждой, от поступков вверх или вниз, а без них —
/// дрейф на уровень к нулю раз в столько дней, сколько задаёт эпоха.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FactionReputation {
    levels: BTreeMap<FactionId, i32>,
    days_since_drift: u32,
}

impl FactionReputation {
    /// Нулевая репутация у каждой фракции контента.
    pub fn new(factions: &BTreeMap<FactionId, Faction>) -> Self {
        Self {
            levels: factions
                .keys()
                .map(|faction| (faction.clone(), 0))
                .collect(),
            days_since_drift: 0,
        }
    }

    pub fn level(&self, faction: &FactionId) -> i32 {
        *self
            .levels
            .get(faction)
            .expect("репутация заведена у каждой фракции контента")
    }

    /// Поступок сдвигает уровень, но не дальше предела правил.
    pub fn change(&mut self, faction: &FactionId, change: i32, rules: ReputationRules) {
        let level = self
            .levels
            .get_mut(faction)
            .expect("репутация заведена у каждой фракции контента");
        *level = rules.clamp(level.saturating_add(change));
    }

    /// Насколько фракция настроена против игрока: деления индикатора угрозы, от нуля до предела.
    pub fn hostility(&self, faction: &FactionId, rules: ReputationRules) -> u8 {
        let hostility = (-self.level(faction)).clamp(0, i32::from(rules.limit));
        u8::try_from(hostility).expect("неприязнь не больше предела правил")
    }

    /// Новый день: раз в `drift_days` дней каждый уровень сдвигается на один к нулю.
    pub fn next_day(&mut self, drift_days: u32) {
        self.days_since_drift += 1;
        if self.days_since_drift < drift_days {
            return;
        }
        self.days_since_drift = 0;
        for level in self.levels.values_mut() {
            *level -= level.signum();
        }
    }
}
