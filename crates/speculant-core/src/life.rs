//! Жизнь (§11): персонаж с биографией, возрастом и бытом по решению 15.4 — скрытым здоровьем и
//! жильём, — финалы, летопись сделок и итоги жизни с открытиями для следующей.

mod character;
mod chronicle;
mod deal_record;
mod ending_kind;
mod good_prices;
mod life_rules;
mod life_summary;
mod pager;
mod pager_rules;
mod price_note;

pub use character::Character;
pub use chronicle::Chronicle;
pub use deal_record::DealRecord;
pub use ending_kind::EndingKind;
pub use good_prices::GoodPrices;
pub use life_rules::{HealthState, LifeRules, NamePools};
pub use life_summary::LifeSummary;
pub use pager::{Pager, PagerMessage};
pub use pager_rules::{PagerDevice, PagerRules};
pub use price_note::PriceNote;
