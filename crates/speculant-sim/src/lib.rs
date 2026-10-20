//! Симулятор баланса (§14.1, пункт 4): боты шести стратегий проживают жизни на настоящем ядре
//! через фасад `Game`, отчёт меряет выживание, капитал, финалы, покрытие событий и тишину эпох,
//! а пороги `[balance]` из `data/balance.toml` говорят, где баланс сломан.

mod balance_check;
mod balance_file;
mod bot;
mod life_outcome;
mod life_plan;
mod life_run;
mod report;
mod simulation;
mod strategy;
#[cfg(test)]
mod test_worlds;
mod trade_report;

pub use balance_check::{CheckError, CheckScope, Violation, arbitrage_edge, check};
pub use balance_file::{
    BALANCE_FILE, BalanceFile, BalanceFileError, BalanceRules, Bounds, CoverageRules, TradeRules,
};
pub use life_outcome::LifeOutcome;
pub use life_plan::LifePlan;
pub use life_run::{horizon, play_life};
pub use report::{
    CapitalReport, Coverage, EndingCount, EpochReport, Report, StrategyCapital, SurvivalYear,
};
pub use simulation::{SimulationSettings, plan_lives, simulate};
pub use strategy::Strategy;
pub use trade_report::{EpochTrade, TradeReport};
