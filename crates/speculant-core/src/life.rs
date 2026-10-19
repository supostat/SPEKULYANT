//! Жизнь (§11): персонаж с биографией, возрастом и бытом по решению 15.4 — скрытым здоровьем и
//! жильём, — финалы, летопись сделок и итоги жизни с открытиями для следующей.

mod character;
mod chronicle;
mod deal_record;
mod ending_kind;
mod life_rules;
mod life_summary;

pub use character::Character;
pub use chronicle::Chronicle;
pub use deal_record::DealRecord;
pub use ending_kind::EndingKind;
pub use life_rules::{HealthState, LifeRules, NamePools};
pub use life_summary::LifeSummary;
