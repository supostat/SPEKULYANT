use serde::Serialize;
use speculant_core::time::{Epoch, GameDate};

use super::CapitalReport;
use crate::LifeOutcome;

/// Дней в среднем году — для жизне-лет.
const DAYS_IN_YEAR: f64 = 365.25;

/// Эпоха глазами прогона: сколько в ней прожито, сколько жизней она сгубила, как часто в ней
/// что-то случалось и с каким капиталом стратегии её закончили.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EpochReport {
    pub epoch: String,
    pub title: String,
    pub life_years: f64,
    /// Жизней, кончившихся в эпохе бедой.
    pub losses: usize,
    /// Бед на жизне-год; `None`, если в эпохе никто не жил.
    pub mortality: Option<f64>,
    pub events: usize,
    /// Дней жизни на одно событие; `None`, если событий не было.
    pub quiet_days: Option<f64>,
    /// Капитал жизней, доживших до конца эпохи; `None`, если таких нет.
    pub capital: Option<CapitalReport>,
}

impl EpochReport {
    /// Эпоха под номером `index` в календаре до первого дня `ends`, не включая его; `lost_on` —
    /// день беды жизни, если она была.
    pub(crate) fn build(
        index: usize,
        epoch: &Epoch,
        ends: GameDate,
        outcomes: &[LifeOutcome],
        lost_on: impl Fn(&LifeOutcome) -> Option<GameDate>,
    ) -> Self {
        let within = |date: GameDate| epoch.starts <= date && date < ends;
        let days: i32 = outcomes
            .iter()
            .map(|outcome| {
                let last = outcome.last_day.min(ends);
                (last.day_index() - epoch.starts.day_index()).max(0)
            })
            .sum();
        let losses = outcomes
            .iter()
            .filter_map(&lost_on)
            .filter(|date| within(*date))
            .count();
        let events = outcomes
            .iter()
            .flat_map(|outcome| &outcome.events)
            .filter(|(date, _)| within(*date))
            .count();
        let capitals: Vec<_> = outcomes
            .iter()
            .filter_map(|outcome| Some((outcome.strategy, *outcome.epoch_capitals.get(index)?)))
            .collect();
        let life_years = f64::from(days) / DAYS_IN_YEAR;
        Self {
            epoch: epoch.id.clone(),
            title: epoch.title.clone(),
            life_years,
            losses,
            mortality: (days > 0).then(|| losses as f64 / life_years),
            events,
            quiet_days: (events > 0).then(|| f64::from(days) / events as f64),
            capital: CapitalReport::build(&capitals),
        }
    }
}
