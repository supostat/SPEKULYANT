//! Отчёт прогона (§14.1): кривая выживания, эпохи — беды и тишина, капитал стратегий, финалы и
//! покрытие событий. Наружу — JSON и текст.

mod capital_report;
mod coverage;
mod ending_count;
mod epoch_report;
mod survival_year;

use std::collections::BTreeMap;
use std::fmt;

use serde::Serialize;
use speculant_core::Content;
use speculant_core::content::EndingId;
use speculant_core::economy::Real;
use speculant_core::life::EndingKind;
use speculant_core::time::GameDate;

pub use capital_report::{CapitalReport, StrategyCapital};
pub use coverage::Coverage;
pub use ending_count::EndingCount;
pub use epoch_report::EpochReport;
pub use survival_year::SurvivalYear;

use crate::LifeOutcome;
use crate::life_run::horizon;

const KOPECKS_PER_RUBLE: i64 = 100;
const DIGIT_GROUP: usize = 3;
const SURVIVAL_YEARS_PER_LINE: usize = 7;

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Report {
    pub lives: usize,
    pub seed: u64,
    /// Первый день бесконечной эпохи: дальше боты не живут.
    pub horizon: GameDate,
    pub survival: Vec<SurvivalYear>,
    /// Эпохи, у которых есть конец, по порядку.
    pub epochs: Vec<EpochReport>,
    /// Капитал на конец жизни — на день финала или первый день бесконечной эпохи.
    pub capital: CapitalReport,
    /// Финалы по убыванию числа жизней.
    pub endings: Vec<EndingCount>,
    /// Жизней, доживших до бесконечной эпохи.
    pub survived: usize,
    pub coverage: Coverage,
}

impl Report {
    pub fn build(content: &Content, seed: u64, outcomes: &[LifeOutcome]) -> Self {
        assert!(!outcomes.is_empty(), "отчёт строят по прожитым жизням");
        let horizon = horizon(content);
        let lost_on = |outcome: &LifeOutcome| {
            let ending = outcome.ending.as_ref()?;
            (content.endings[ending].kind != EndingKind::Departure).then_some(outcome.last_day)
        };
        let epochs = content
            .time
            .epochs
            .epochs()
            .windows(2)
            .enumerate()
            .map(|(index, pair)| {
                EpochReport::build(index, &pair[0], pair[1].starts, outcomes, lost_on)
            })
            .collect();
        let final_capitals: Vec<_> = outcomes
            .iter()
            .map(|outcome| (outcome.strategy, outcome.capital))
            .collect();
        let survival = SurvivalYear::curve(
            GameDate::START.year()..horizon.year(),
            outcomes,
            |outcome| lost_on(outcome).map(GameDate::year),
        );
        Self {
            lives: outcomes.len(),
            seed,
            horizon,
            survival,
            epochs,
            capital: CapitalReport::build(&final_capitals).expect("жизни прожиты — проверено выше"),
            endings: ending_counts(content, outcomes),
            survived: outcomes
                .iter()
                .filter(|outcome| outcome.ending.is_none())
                .count(),
            coverage: Coverage::build(content, seed, outcomes),
        }
    }
}

fn ending_counts(content: &Content, outcomes: &[LifeOutcome]) -> Vec<EndingCount> {
    let mut lives: BTreeMap<&EndingId, usize> = BTreeMap::new();
    for ending in outcomes
        .iter()
        .filter_map(|outcome| outcome.ending.as_ref())
    {
        *lives.entry(ending).or_default() += 1;
    }
    let mut counts: Vec<EndingCount> = lives
        .into_iter()
        .map(|(ending, lives)| EndingCount {
            ending: ending.clone(),
            title: content.endings[ending].title.clone(),
            lives,
        })
        .collect();
    counts.sort_by(|left, right| right.lives.cmp(&left.lives));
    counts
}

impl fmt::Display for Report {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            formatter,
            "Прогон: жизней {}, сид {}, история до {}",
            self.lives,
            self.seed,
            self.horizon.long()
        )?;
        writeln!(formatter, "Выживание к концу года:")?;
        for line in self.survival.chunks(SURVIVAL_YEARS_PER_LINE) {
            let years: Vec<String> = line
                .iter()
                .map(|point| format!("{} {:.0}%", point.year, point.alive * 100.0))
                .collect();
            writeln!(formatter, "  {}", years.join("  "))?;
        }
        writeln!(formatter, "Эпохи:")?;
        for epoch in &self.epochs {
            let mortality = epoch.mortality.map_or_else(
                || "никто не жил".to_owned(),
                |rate| format!("{rate:.4} в год"),
            );
            let quiet = epoch.quiet_days.map_or_else(
                || "событий нет".to_owned(),
                |days| format!("событие раз в {days:.1} дн."),
            );
            writeln!(
                formatter,
                "  {}: жизне-лет {:.1}, бед {} ({mortality}), событий {} ({quiet})",
                epoch.title, epoch.life_years, epoch.losses, epoch.events
            )?;
            if let Some(capital) = &epoch.capital {
                write_capital(formatter, "к концу эпохи", capital)?;
            }
        }
        write_capital(formatter, "на конец жизни", &self.capital)?;
        let mut endings: Vec<String> = self
            .endings
            .iter()
            .map(|ending| format!("{} — {}", ending.title, ending.lives))
            .collect();
        endings.push(format!("дожили — {}", self.survived));
        writeln!(formatter, "Финалы: {}", endings.join(", "))?;
        let never: Vec<&str> = self
            .coverage
            .never_fired()
            .map(|event| event.as_str())
            .collect();
        writeln!(
            formatter,
            "Покрытие: сработали {} событий из {}, всё покрытие — к жизни {}",
            self.coverage.events.len() - never.len(),
            self.coverage.events.len(),
            self.coverage.saturated_at
        )?;
        if !never.is_empty() {
            writeln!(formatter, "  ни разу: {}", never.join(", "))?;
        }
        Ok(())
    }
}

/// Капитал на момент: медиана всех и строка на стратегию.
fn write_capital(
    formatter: &mut fmt::Formatter<'_>,
    moment: &str,
    capital: &CapitalReport,
) -> fmt::Result {
    writeln!(
        formatter,
        "    капитал {moment}, р. 1984 года, медиана всех — {}:",
        rubles(capital.median)
    )?;
    for strategy in &capital.strategies {
        let medians = strategy
            .medians
            .map_or_else(|| "—".to_owned(), |medians| format!("{medians:.2}"));
        writeln!(
            formatter,
            "      {:<13} жизней {:>4}: 10% — {}, медиана — {}, 90% — {}; медиан всех: {medians}",
            strategy.strategy.id(),
            strategy.lives,
            rubles(strategy.low),
            rubles(strategy.median),
            rubles(strategy.high)
        )?;
    }
    Ok(())
}

/// Целые рубли с разбивкой по три цифры: «12 400».
fn rubles(real: Real) -> String {
    let rubles = real.kopecks() / KOPECKS_PER_RUBLE;
    let digits = rubles.unsigned_abs().to_string();
    let groups: Vec<&str> = digits
        .as_bytes()
        .rchunks(DIGIT_GROUP)
        .rev()
        .map(|group| std::str::from_utf8(group).expect("цифры — ASCII"))
        .collect();
    let sign = if rubles < 0 { "-" } else { "" };
    format!("{sign}{}", groups.join(" "))
}
