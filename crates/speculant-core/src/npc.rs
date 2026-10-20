//! Соперники (§8): 10–15 постоянных персонажей на мир из пула шаблонов по сиду. Они торгуют по
//! тем же правилам, что игрок, только грубее, живут своей судьбой и помнят игроку обиды и услуги.

mod fate;
mod fate_kind;
mod rival;
mod rival_deal;
mod rival_rules;
mod rivals;
mod strategy;
mod strategy_rules;

pub use fate::Fate;
pub use fate_kind::FateKind;
pub use rival::Rival;
pub use rival_deal::{DealSide, RivalDeal};
pub use rival_rules::RivalRules;
pub use rivals::Rivals;
pub use strategy::Strategy;
pub use strategy_rules::StrategyRules;
