use std::sync::Arc;
use std::thread;

use anyhow::{Result, anyhow, bail};
use speculant_core::Content;
use speculant_sim::{BalanceFile, Report, SimulationSettings, Strategy, simulate};
use speculant_validate::{coverage, strict};

use crate::project_root::project_root;

/// Откуда покрытие в ошибках про сам прогон.
const RUN: &str = "прогон coverage-check";

/// `cargo xtask coverage-check`: прогон `[coverage] lives` жизней всеми стратегиями с сидом
/// `[balance] seed` на настоящем ядре и данных, строгая проверка `data/` и сверка покрытия —
/// каждое событие хоть раз сработало или помечено `rare = true`.
pub fn run() -> Result<()> {
    let data = project_root().join("data");
    let not_loaded = |errors: Vec<_>| {
        anyhow!(
            "data/ не загружается ({} ошибок) — сначала cargo run -p speculant-validate -- data",
            errors.len()
        )
    };
    let files = Content::read_files(&data).map_err(not_loaded)?;
    let content = Arc::new(Content::from_files(files.clone()).map_err(not_loaded)?);
    let rules = BalanceFile::read(&data)?;
    let settings = SimulationSettings {
        lives: rules.coverage.lives.get(),
        seed: rules.balance.seed,
        strategies: Strategy::ALL.to_vec(),
        threads: thread::available_parallelism().map_or(1, usize::from),
        curiosity: rules.coverage.curiosity,
    };
    let report = Report::build(&content, settings.seed, &simulate(&content, &settings));
    let mut errors = strict::check(&content, &files);
    errors.extend(coverage::check(
        &content,
        &files,
        &report.coverage,
        RUN,
        &rules.coverage,
    ));
    if !errors.is_empty() {
        for error in &errors {
            eprintln!("  {error}");
        }
        bail!(
            "coverage-check: ошибок — {} (жизней {}, сид {})",
            errors.len(),
            settings.lives,
            settings.seed
        );
    }
    println!(
        "coverage-check: {}; события достижимы, мёртвых цепочек нет (сид {})",
        coverage::summary(&content, &report.coverage),
        settings.seed
    );
    Ok(())
}
