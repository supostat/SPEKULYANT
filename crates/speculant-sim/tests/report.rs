//! Метрики отчёта на жизнях с известным исходом: кривая выживания, беды и тишина эпох, капитал
//! стратегий к концу эпох и жизни, финалы и покрытие событий.

use std::path::Path;

use speculant_core::Content;
use speculant_core::content::{BiographyId, EndingId, EventId};
use speculant_core::economy::Real;
use speculant_core::time::GameDate;
use speculant_sim::{LifeOutcome, Report, Strategy};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
const EVENTS_FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../speculant-core/tests/fixtures/events"
);
const RAIKOM: &str = "raikom_offer";

fn content() -> Content {
    Content::load(Path::new(DATA)).expect("data/ валиден")
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn rubles(rubles: i64) -> Real {
    Real::from_kopecks(rubles * 100)
}

fn life(
    strategy: Strategy,
    ending: Option<&str>,
    last_day: GameDate,
    capitals: &[i64],
    events: &[GameDate],
) -> LifeOutcome {
    let (final_capital, epoch_capitals) = capitals.split_last().expect("капитал на конец жизни");
    LifeOutcome {
        strategy,
        biography: BiographyId::new("student"),
        seed: 0,
        ending: ending.map(EndingId::new),
        last_day,
        capital: rubles(*final_capital),
        epoch_capitals: epoch_capitals
            .iter()
            .map(|capital| rubles(*capital))
            .collect(),
        events: events
            .iter()
            .map(|date| (*date, EventId::new(RAIKOM)))
            .collect(),
    }
}

/// Убит в 1990-м, эмигрировал в 1991-м, дожил с тремя событиями застоя, сел навсегда в 1993-м.
fn outcomes() -> Vec<LifeOutcome> {
    vec![
        life(
            Strategy::Random,
            Some("death_bandits"),
            date(1990, 6, 1),
            &[500, 100],
            &[],
        ),
        life(
            Strategy::Arbitrage,
            Some("emigration"),
            date(1991, 3, 1),
            &[700, 300],
            &[],
        ),
        life(
            Strategy::Hoarder,
            None,
            date(2005, 1, 1),
            &[900, 2000, 3000, 4000, 1000],
            &[date(1984, 5, 1), date(1985, 5, 1), date(1986, 5, 1)],
        ),
        life(
            Strategy::Cautious,
            Some("prison"),
            date(1993, 1, 1),
            &[100, 200, 50],
            &[],
        ),
    ]
}

fn report() -> Report {
    Report::build(&content(), 1, &outcomes())
}

#[test]
fn survival_counts_death_and_prison_as_losses_but_not_emigration() {
    let survival = report().survival;
    let alive = |year: i32| {
        survival
            .iter()
            .find(|point| point.year == year)
            .expect("год кривой")
            .alive
    };
    assert_eq!(survival.first().map(|point| point.year), Some(1984));
    assert_eq!(survival.last().map(|point| point.year), Some(2004));
    assert_eq!(alive(1989), 1.0);
    assert_eq!(alive(1990), 0.75);
    assert_eq!(alive(1991), 0.75, "эмиграция — уход по своей воле");
    assert_eq!(alive(1993), 0.5);
    assert_eq!(alive(2004), 0.5);
}

#[test]
fn an_epoch_counts_its_life_years_losses_and_days_per_event() {
    let report = report();
    let ids: Vec<&str> = report
        .epochs
        .iter()
        .map(|epoch| epoch.epoch.as_str())
        .collect();
    assert_eq!(
        ids,
        ["stagnation", "perestroika", "wild_market", "stabilization"]
    );
    let stagnation = &report.epochs[0];
    assert_eq!(stagnation.life_years, 4.0 * 1461.0 / 365.25);
    assert_eq!(stagnation.losses, 0);
    assert_eq!(stagnation.events, 3);
    assert_eq!(stagnation.quiet_days, Some(4.0 * 1461.0 / 3.0));
    let perestroika = &report.epochs[1];
    let days = 882.0 + 1155.0 + 2.0 * 1461.0;
    assert_eq!(perestroika.losses, 1);
    assert_eq!(perestroika.mortality, Some(1.0 / (days / 365.25)));
    assert_eq!(perestroika.quiet_days, None, "событий не было");
    assert_eq!(
        report.epochs[2].losses, 1,
        "срок, который не пересидеть, — беда"
    );
}

#[test]
fn capital_compares_each_strategy_median_with_the_median_of_all() {
    let report = report();
    assert_eq!(report.capital.median, rubles(100));
    let hoarder = report
        .capital
        .strategies
        .iter()
        .find(|capital| capital.strategy == Strategy::Hoarder)
        .expect("знаток жил");
    assert_eq!((hoarder.lives, hoarder.median), (1, rubles(1000)));
    assert_eq!(hoarder.medians, Some(10.0));
    let stagnation = report.epochs[0]
        .capital
        .as_ref()
        .expect("до 1988-го дожили все");
    assert_eq!(stagnation.median, rubles(500));
    let perestroika = report.epochs[1]
        .capital
        .as_ref()
        .expect("до 1992-го дожили двое");
    let lives: Vec<Strategy> = perestroika
        .strategies
        .iter()
        .map(|capital| capital.strategy)
        .collect();
    assert_eq!(lives, [Strategy::Hoarder, Strategy::Cautious]);
}

#[test]
fn capital_percentiles_go_by_the_nearest_rank() {
    let hoarders: Vec<LifeOutcome> = [10, 20, 30, 40]
        .map(|capital| {
            life(
                Strategy::Hoarder,
                None,
                date(2005, 1, 1),
                &[0, 0, 0, 0, capital],
                &[],
            )
        })
        .to_vec();
    let report = Report::build(&content(), 1, &hoarders);
    let hoarder = &report.capital.strategies[0];
    assert_eq!(
        (hoarder.low, hoarder.median, hoarder.high),
        (rubles(10), rubles(20), rubles(40)),
        "из четырёх: десятый процентиль — первый, медиана — второй, девяностый — четвёртый"
    );
}

#[test]
fn coverage_is_saturated_at_the_life_where_the_last_new_event_fired() {
    let content = Content::load(Path::new(EVENTS_FIXTURE)).expect("фикстура событий валидна");
    let fired = |events: &[&str]| LifeOutcome {
        events: events
            .iter()
            .map(|event| (date(1985, 1, 1), EventId::new(*event)))
            .collect(),
        ..life(Strategy::Random, None, date(1988, 1, 1), &[0, 0], &[])
    };
    let outcomes = [
        fired(&["queue"]),
        fired(&["rain", "queue"]),
        fired(&["queue"]),
    ];
    let coverage = Report::build(&content, 1, &outcomes).coverage;
    assert_eq!(coverage.saturated_at, 2, "дождь впервые — во второй жизни");
    assert_eq!(coverage.events[&EventId::new("queue")], 3);
    assert!(
        coverage
            .never_fired()
            .any(|event| event.as_str() == "berezka")
    );
}

#[test]
fn endings_and_coverage_are_counted_over_all_lives() {
    let report = report();
    let endings: Vec<(&str, usize)> = report
        .endings
        .iter()
        .map(|count| (count.ending.as_str(), count.lives))
        .collect();
    assert_eq!(
        endings,
        [("death_bandits", 1), ("emigration", 1), ("prison", 1)]
    );
    assert_eq!(report.survived, 1);
    assert_eq!(report.coverage.lives, 4);
    assert_eq!(report.coverage.events[&EventId::new(RAIKOM)], 3);
    assert_eq!(
        report.coverage.saturated_at, 3,
        "событие впервые — в третьей жизни"
    );
    assert_eq!(report.coverage.never_fired().count(), 0);
}

#[test]
fn the_report_goes_out_as_json_with_rubles_and_iso_dates() {
    let json = serde_json::to_value(report()).expect("отчёт сериализуется");
    assert_eq!(json["horizon"], "2005-01-01");
    assert_eq!(json["capital"]["median"], 100.0);
    assert_eq!(json["capital"]["strategies"][0]["strategy"], "random");
    assert_eq!(json["coverage"]["events"][RAIKOM], 3);
}
