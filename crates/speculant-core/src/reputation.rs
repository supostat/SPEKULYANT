//! Репутация и связи (§9, решение 15.6): уровни по шести фракциям с дрейфом к нулю в темпе эпохи
//! и люди с именами, которых можно попросить, которым можно задолжать и которых можно потерять.

mod availability;
mod contact;
mod contact_refusal;
mod faction_reputation;
mod reputation_rules;

pub use availability::Availability;
pub use contact::Contact;
pub use contact_refusal::ContactRefusal;
pub use faction_reputation::FactionReputation;
pub use reputation_rules::ReputationRules;
