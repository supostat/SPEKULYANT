use serde::{Deserialize, Serialize};

use crate::content::{EmployeeTraits, HabitId, NpcId};

/// Нанятый человек (§7.2): черты из шаблона на день найма — дальше они живут своей жизнью,
/// хватку можно пропить.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Employee {
    npc: NpcId,
    skill: u8,
    honesty: u8,
    loyalty: u8,
    habits: Vec<HabitId>,
}

impl Employee {
    pub fn from_template(npc: NpcId, traits: &EmployeeTraits) -> Self {
        Self {
            npc,
            skill: traits.skill,
            honesty: traits.honesty,
            loyalty: traits.loyalty,
            habits: traits.habits.clone(),
        }
    }

    pub fn npc(&self) -> &NpcId {
        &self.npc
    }

    /// Хватка — навык вести дело.
    pub fn skill(&self) -> u8 {
        self.skill
    }

    pub fn honesty(&self) -> u8 {
        self.honesty
    }

    pub fn loyalty(&self) -> u8 {
        self.loyalty
    }

    pub fn habits(&self) -> &[HabitId] {
        &self.habits
    }

    /// Деление хватки ушло; ниже первого не падает.
    pub fn lose_skill(&mut self) {
        self.skill = self.skill.saturating_sub(1).max(1);
    }
}
