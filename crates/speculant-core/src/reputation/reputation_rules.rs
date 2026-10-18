use serde::Deserialize;

/// Правила репутации и связей из `data/reputation.toml`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReputationRules {
    /// Уровни репутации и отношения к человеку — от `-limit` до `limit`; столько же делений у
    /// индикатора угрозы в «Кармане».
    pub limit: u8,
    /// Сколько невозвращённых услуг человек терпит, прежде чем перестать помогать.
    pub max_debt: u32,
}

impl ReputationRules {
    /// Уровень в пределах `-limit..=limit`.
    pub fn clamp(self, level: i32) -> i32 {
        let limit = i32::from(self.limit);
        level.clamp(-limit, limit)
    }
}
