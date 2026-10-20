use std::sync::Arc;
use std::thread;

use anyhow::{Result, anyhow, bail};
use speculant_core::Content;
use speculant_sim::{
    BalanceFile, CheckScope, Report, SimulationSettings, Strategy, TradeReport, arbitrage_edge,
    check, simulate,
};

use crate::project_root::project_root;

/// `cargo xtask balance-check <эпоха|all>`: прогон `[balance] lives` жизней всеми стратегиями с
/// сидом `[balance] seed` на настоящем ядре и данных, замер выгоды рейсов с кошельком
/// `[trade] purse` и сверка эпохи или всех эпох с порогами `[balance]` и `[trade]` из
/// `data/balance.toml` — беды, тишина, стратегии, ломающие экономику, и окупаемость торговли.
pub fn run(scope: &str) -> Result<()> {
    let data = project_root().join("data");
    let content = Content::load(&data).map_err(|errors| {
        anyhow!(
            "data/ не загружается ({} ошибок) — сначала cargo run -p speculant-validate -- data",
            errors.len()
        )
    })?;
    let content = Arc::new(content);
    let rules = BalanceFile::read(&data)?;
    let settings = SimulationSettings {
        lives: rules.balance.lives.get(),
        seed: rules.balance.seed,
        strategies: Strategy::ALL.to_vec(),
        threads: thread::available_parallelism().map_or(1, usize::from),
        curiosity: 0.0,
    };
    let report = Report::build(&content, settings.seed, &simulate(&content, &settings));
    let trade = TradeReport::build(&content, rules.trade.purse);
    let violations = check(&report, &trade, &rules, &CheckScope::parse(scope))?;
    for epoch in &report.epochs {
        let trip = trade.epoch(&epoch.epoch).map(|trade| trade.trip_return);
        let edge = arbitrage_edge(epoch);
        println!(
            "  {}: выгода лучшего рейса {}, арбитражник богаче случайного бота в {}",
            epoch.epoch,
            trip.map_or_else(|| "—".to_owned(), |share| format!("{:.1}%", share * 100.0)),
            edge.map_or_else(|| "—".to_owned(), |edge| format!("{edge:.1} раза"))
        );
    }
    if !violations.is_empty() {
        for violation in &violations {
            eprintln!("  {violation}");
        }
        bail!(
            "balance-check {scope}: нарушено порогов — {} (жизней {}, сид {})",
            violations.len(),
            settings.lives,
            settings.seed
        );
    }
    println!(
        "balance-check {scope}: в порогах (жизней {}, сид {})",
        settings.lives, settings.seed
    );
    Ok(())
}
