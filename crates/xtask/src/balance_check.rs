use std::sync::Arc;
use std::thread;

use anyhow::{Result, anyhow, bail};
use speculant_core::Content;
use speculant_sim::{
    BalanceFile, CheckScope, Report, SimulationSettings, Strategy, check, simulate,
};

use crate::project_root::project_root;

/// `cargo xtask balance-check <эпоха|all>`: прогон `[balance] lives` жизней всеми стратегиями с
/// сидом `[balance] seed` на настоящем ядре и данных и сверка эпохи или всех эпох с порогами
/// `[balance]` из `data/balance.toml` — беды, тишина и стратегии, ломающие экономику.
pub fn run(scope: &str) -> Result<()> {
    let data = project_root().join("data");
    let content = Content::load(&data).map_err(|errors| {
        anyhow!(
            "data/ не загружается ({} ошибок) — сначала cargo run -p speculant-validate -- data",
            errors.len()
        )
    })?;
    let content = Arc::new(content);
    let rules = BalanceFile::read(&data)?.balance;
    let settings = SimulationSettings {
        lives: rules.lives.get(),
        seed: rules.seed,
        strategies: Strategy::ALL.to_vec(),
        threads: thread::available_parallelism().map_or(1, usize::from),
    };
    let report = Report::build(&content, settings.seed, &simulate(&content, &settings));
    let violations = check(&report, &rules, &CheckScope::parse(scope))?;
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
