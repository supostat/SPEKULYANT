//! Пороги `[balance]` и `[coverage]` из `data/balance.toml` и сверка с ними прогона: беды и тишина
//! эпохи, стратегии, ломающие экономику к концу эпохи; одна эпоха или все.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::path::Path;

use speculant_core::Content;
use speculant_core::content::{BiographyId, EndingId, EventId};
use speculant_core::economy::Real;
use speculant_core::time::GameDate;
use speculant_sim::{
    BalanceFile, BalanceRules, CheckError, CheckScope, LifeOutcome, Report, Strategy, Violation,
    check,
};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
const EPOCHS: [&str; 4] = ["stagnation", "perestroika", "wild_market", "stabilization"];

fn content() -> Content {
    Content::load(Path::new(DATA)).expect("data/ валиден")
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

/// Две жизни: случайный игрок убит в 1986-м, знаток доживает с капиталом вдесятеро больше и
/// встречает по событию в каждой эпохе.
fn report() -> Report {
    let outcomes = [
        LifeOutcome {
            strategy: Strategy::Random,
            biography: BiographyId::new("student"),
            seed: 0,
            ending: Some(EndingId::new("death_bandits")),
            last_day: date(1986, 1, 1),
            capital: Real::from_kopecks(10_000),
            epoch_capitals: Vec::new(),
            events: Vec::new(),
        },
        LifeOutcome {
            strategy: Strategy::Hoarder,
            biography: BiographyId::new("student"),
            seed: 0,
            ending: None,
            last_day: date(2005, 1, 1),
            capital: Real::from_kopecks(100_000),
            epoch_capitals: vec![Real::from_kopecks(100_000); 4],
            events: [1985, 1990, 1995, 2000]
                .map(|year| (date(year, 3, 1), EventId::new("raikom_offer")))
                .to_vec(),
        },
    ];
    Report::build(&content(), 1, &outcomes)
}

fn each_epoch(value: f64) -> BTreeMap<String, f64> {
    EPOCHS
        .iter()
        .map(|epoch| ((*epoch).to_owned(), value))
        .collect()
}

fn rules() -> BalanceRules {
    BalanceRules {
        seed: 1,
        lives: NonZeroUsize::new(2).expect("две жизни"),
        breaking_medians: 100.0,
        mortality: each_epoch(1.0),
        quiet_days: each_epoch(f64::INFINITY),
    }
}

#[test]
fn the_balance_file_of_the_data_carries_calibrated_thresholds_for_every_epoch() {
    let file = BalanceFile::read(Path::new(DATA)).expect("data/balance.toml разбирается");
    for epoch in EPOCHS {
        assert!(
            file.balance.mortality.contains_key(epoch),
            "{epoch}: порог бед"
        );
        assert!(
            file.balance.quiet_days.contains_key(epoch),
            "{epoch}: порог тишины"
        );
    }
    assert!(
        file.balance.breaking_medians > 1.0,
        "стратегия выше общей медианы ещё не ломает"
    );
    assert!(file.coverage.lives >= file.balance.lives);
}

#[test]
fn a_run_of_no_lives_is_refused() {
    let text = "[balance]\nseed = 1\nlives = 0\nbreaking_medians = 3.0\n\
                [balance.mortality]\n[balance.quiet_days]\n[coverage]\nlives = 5\n";
    assert!(BalanceFile::parse(text).is_err());
}

#[test]
fn an_unknown_threshold_in_the_balance_file_is_refused() {
    let text = "[balance]\nseed = 1\nlives = 2\nbreaking_medians = 3.0\nbandits = 1\n\
                [balance.mortality]\n[balance.quiet_days]\n[coverage]\nlives = 5\n";
    assert!(BalanceFile::parse(text).is_err());
}

#[test]
fn a_run_inside_every_threshold_has_no_violations() {
    assert_eq!(check(&report(), &rules(), &CheckScope::All), Ok(Vec::new()));
}

#[test]
fn deaths_above_the_epoch_threshold_are_a_violation_of_that_epoch_only() {
    let strict = BalanceRules {
        mortality: each_epoch(0.1),
        ..rules()
    };
    let violations = check(&report(), &strict, &CheckScope::All).expect("пороги есть");
    let epochs: Vec<&str> = violations
        .iter()
        .map(|violation| match violation {
            Violation::Mortality { epoch, .. } => epoch.as_str(),
            other => panic!("только беды: {other:?}"),
        })
        .collect();
    assert_eq!(epochs, ["stagnation"]);
    let perestroika = CheckScope::Epoch("perestroika".to_owned());
    assert_eq!(check(&report(), &strict, &perestroika), Ok(Vec::new()));
}

#[test]
fn rare_events_are_a_violation_and_an_epoch_without_any_is_one_whatever_the_threshold() {
    let scope = CheckScope::Epoch("wild_market".to_owned());
    let busy = BalanceRules {
        quiet_days: each_epoch(365.0),
        ..rules()
    };
    let violations = check(&report(), &busy, &scope).expect("пороги есть");
    assert!(matches!(
        violations[..],
        [Violation::Quiet { measured: Some(days), .. }] if days > 365.0
    ));
    let mut quiet = report();
    quiet.epochs[2].events = 0;
    quiet.epochs[2].quiet_days = None;
    let violations = check(&quiet, &rules(), &scope).expect("пороги есть");
    assert_eq!(
        violations,
        [Violation::Quiet {
            epoch: "wild_market".to_owned(),
            measured: None,
            limit: f64::INFINITY
        }]
    );
}

#[test]
fn a_strategy_far_above_the_median_breaks_the_economy_at_the_epoch_end() {
    let greedy = BalanceRules {
        breaking_medians: 0.5,
        quiet_days: each_epoch(f64::INFINITY),
        ..rules()
    };
    let scope = CheckScope::Epoch("stabilization".to_owned());
    let violations = check(&report(), &greedy, &scope).expect("пороги есть");
    assert!(violations.iter().any(|violation| matches!(
        violation,
        Violation::Breaking {
            strategy: Strategy::Hoarder,
            ..
        }
    )));
}

#[test]
fn an_unknown_epoch_or_a_missing_threshold_cannot_be_checked() {
    assert!(matches!(
        check(
            &report(),
            &rules(),
            &CheckScope::Epoch("sandbox".to_owned())
        ),
        Err(CheckError::UnknownEpoch { .. })
    ));
    let mut partial = rules();
    partial.quiet_days.remove("perestroika");
    assert_eq!(
        check(&report(), &partial, &CheckScope::All),
        Err(CheckError::MissingThreshold {
            epoch: "perestroika".to_owned(),
            section: "[balance.quiet_days]"
        })
    );
}
