use serde::{Deserialize, Serialize};

use crate::content::{BiographyId, HousingId};
use crate::time::GameDate;

/// Персонаж жизни (§11, решение 15.4): имя, биография, день рождения, скрытая шкала здоровья и
/// жильё. Числовых навыков у него нет — сила в ресурсах, связях и репутации.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Character {
    name: String,
    biography: BiographyId,
    born: GameDate,
    health: i32,
    housing: HousingId,
}

impl Character {
    pub fn new(
        name: String,
        biography: BiographyId,
        born: GameDate,
        health: i32,
        housing: HousingId,
    ) -> Self {
        Self {
            name,
            biography,
            born,
            health,
            housing,
        }
    }

    /// «Лыков Пётр Сергеевич».
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn biography(&self) -> &BiographyId {
        &self.biography
    }

    pub fn born(&self) -> GameDate {
        self.born
    }

    pub fn age_on(&self, date: GameDate) -> u32 {
        date.age_on(self.born)
    }

    /// Сегодня ему исполнилось на год больше, чем вчера.
    pub fn has_birthday(&self, date: GameDate) -> bool {
        self.age_on(date) > self.age_on(date.previous())
    }

    /// Скрытая шкала здоровья; на нуле и ниже жизнь кончается.
    pub fn health(&self) -> i32 {
        self.health
    }

    /// Здоровье от лет, пьянства и событий — не выше `max`.
    pub fn change_health(&mut self, change: i32, max: i32) {
        self.health = self.health.saturating_add(change).min(max);
    }

    pub fn housing(&self) -> &HousingId {
        &self.housing
    }

    pub fn move_to(&mut self, housing: HousingId) {
        self.housing = housing;
    }
}
