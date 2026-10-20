//! Пороги `[balance]`, `[trade]` и `[coverage]` из `data/balance.toml` и сверка с ними прогона:
//! беды и тишина эпохи, стратегии, ломающие экономику к концу эпохи, выгода лучшего рейса и
//! перевес перекупщика над случайным ботом; одна эпоха или все.

use std::collections::BTreeMap;
use std::num::NonZeroUsize;
use std::path::Path;

use speculant_core::Content;
use speculant_core::content::{BiographyId, EndingId, EventId};
use speculant_core::economy::Real;
use speculant_core::time::GameDate;
use speculant_sim::{
    BalanceFile, BalanceRules, Bounds, CheckError, CheckScope, CoverageRules, EpochTrade,
    LifeOutcome, Report, Strategy, TradeReport, TradeRules, Violation, check,
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

fn each_epoch<T: Clone>(value: T) -> BTreeMap<String, T> {
    EPOCHS
        .iter()
        .map(|epoch| ((*epoch).to_owned(), value.clone()))
        .collect()
}

const ANY: Bounds = Bounds {
    from: f64::NEG_INFINITY,
    to: f64::INFINITY,
};

fn rules() -> BalanceFile {
    BalanceFile {
        balance: BalanceRules {
            seed: 1,
            lives: NonZeroUsize::new(2).expect("две жизни"),
            breaking_medians: 100.0,
            mortality: each_epoch(1.0),
            quiet_days: each_epoch(f64::INFINITY),
        },
        trade: TradeRules {
            purse: Real::from_kopecks(50_000),
            trip_return: ANY,
            arbitrage_over_random: each_epoch(ANY),
        },
        coverage: CoverageRules {
            lives: NonZeroUsize::new(2).expect("две жизни"),
            curiosity: 0.1,
        },
    }
}

/// Лучший рейс в каждой эпохе приносит треть кошелька.
fn trade() -> TradeReport {
    TradeReport {
        epochs: EPOCHS
            .iter()
            .map(|epoch| EpochTrade {
                epoch: (*epoch).to_owned(),
                title: (*epoch).to_owned(),
                weeks: 52,
                trip_return: 0.3,
            })
            .collect(),
    }
}

const TRADE_SECTION: &str = "[trade]\npurse = 500\ntrip_return = { from = 0.2, to = 0.6 }\n\
                             [trade.arbitrage_over_random]\n";

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
    for epoch in EPOCHS {
        let edge = file.trade.arbitrage_over_random[epoch];
        assert!(
            1.0 < edge.from && edge.from < edge.to,
            "{epoch}: перекупщик богаче случайного бота"
        );
    }
    assert!(file.trade.purse.is_positive());
    assert!(
        0.0 < file.trade.trip_return.from
            && file.trade.trip_return.from < file.trade.trip_return.to
    );
    assert!(file.coverage.lives >= file.balance.lives);
    assert!(
        file.coverage.curiosity > 0.0 && file.coverage.curiosity < 1.0,
        "боты покрытия любопытны, но не играют наугад"
    );
}

#[test]
fn a_run_of_no_lives_is_refused() {
    let text = format!(
        "[balance]\nseed = 1\nlives = 0\nbreaking_medians = 3.0\n\
         [balance.mortality]\n[balance.quiet_days]\n{TRADE_SECTION}[coverage]\nlives = 5\n\
         curiosity = 0.1\n"
    );
    assert!(BalanceFile::parse(&text).is_err());
}

#[test]
fn an_unknown_threshold_in_the_balance_file_is_refused() {
    let text = format!(
        "[balance]\nseed = 1\nlives = 2\nbreaking_medians = 3.0\nbandits = 1\n\
         [balance.mortality]\n[balance.quiet_days]\n{TRADE_SECTION}[coverage]\nlives = 5\n\
         curiosity = 0.1\n"
    );
    assert!(BalanceFile::parse(&text).is_err());
}

#[test]
fn curiosity_beyond_a_share_is_refused() {
    let text = format!(
        "[balance]\nseed = 1\nlives = 2\nbreaking_medians = 3.0\n\
         [balance.mortality]\n[balance.quiet_days]\n{TRADE_SECTION}[coverage]\nlives = 5\n\
         curiosity = 1.5\n"
    );
    assert_eq!(
        BalanceFile::parse(&text).map_err(|error| error.to_string()),
        Err(
            "balance.toml не разбирается: [coverage] curiosity 1.5: нужна доля от 0 до 1"
                .to_owned()
        )
    );
}

#[test]
fn a_run_inside_every_threshold_has_no_violations() {
    assert_eq!(
        check(&report(), &trade(), &rules(), &CheckScope::All),
        Ok(Vec::new())
    );
}

#[test]
fn deaths_above_the_epoch_threshold_are_a_violation_of_that_epoch_only() {
    let mut strict = rules();
    strict.balance.mortality = each_epoch(0.1);
    let violations = check(&report(), &trade(), &strict, &CheckScope::All).expect("пороги есть");
    let epochs: Vec<&str> = violations
        .iter()
        .map(|violation| match violation {
            Violation::Mortality { epoch, .. } => epoch.as_str(),
            other => panic!("только беды: {other:?}"),
        })
        .collect();
    assert_eq!(epochs, ["stagnation"]);
    let perestroika = CheckScope::Epoch("perestroika".to_owned());
    assert_eq!(
        check(&report(), &trade(), &strict, &perestroika),
        Ok(Vec::new())
    );
}

#[test]
fn rare_events_are_a_violation_and_an_epoch_without_any_is_one_whatever_the_threshold() {
    let scope = CheckScope::Epoch("wild_market".to_owned());
    let mut busy = rules();
    busy.balance.quiet_days = each_epoch(365.0);
    let violations = check(&report(), &trade(), &busy, &scope).expect("пороги есть");
    assert!(matches!(
        violations[..],
        [Violation::Quiet { measured: Some(days), .. }] if days > 365.0
    ));
    let mut quiet = report();
    quiet.epochs[2].events = 0;
    quiet.epochs[2].quiet_days = None;
    let violations = check(&quiet, &trade(), &rules(), &scope).expect("пороги есть");
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
    let mut greedy = rules();
    greedy.balance.breaking_medians = 0.5;
    let scope = CheckScope::Epoch("stabilization".to_owned());
    let violations = check(&report(), &trade(), &greedy, &scope).expect("пороги есть");
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
            &trade(),
            &rules(),
            &CheckScope::Epoch("sandbox".to_owned())
        ),
        Err(CheckError::UnknownEpoch { .. })
    ));
    let mut partial = rules();
    partial.balance.quiet_days.remove("perestroika");
    assert_eq!(
        check(&report(), &trade(), &partial, &CheckScope::All),
        Err(CheckError::MissingThreshold {
            epoch: "perestroika".to_owned(),
            section: "[balance.quiet_days]"
        })
    );
    let mut no_edge = rules();
    no_edge.trade.arbitrage_over_random.remove("perestroika");
    assert_eq!(
        check(&report(), &trade(), &no_edge, &CheckScope::All),
        Err(CheckError::MissingThreshold {
            epoch: "perestroika".to_owned(),
            section: "[trade.arbitrage_over_random]"
        })
    );
}

#[test]
fn a_best_trip_returning_outside_the_bounds_is_a_violation_both_ways() {
    let scope = CheckScope::Epoch("perestroika".to_owned());
    let mut picky = rules();
    picky.trade.trip_return = Bounds { from: 0.2, to: 0.6 };
    for (margin, kind) in [(0.05, "не окупается"), (0.9, "слишком легко")] {
        let mut measured = trade();
        measured.epochs[1].trip_return = margin;
        assert_eq!(
            check(&report(), &measured, &picky, &scope),
            Ok(vec![Violation::TripReturn {
                epoch: "perestroika".to_owned(),
                measured: margin,
                bounds: picky.trade.trip_return,
            }]),
            "{kind}"
        );
    }
}

/// Случайный бот и арбитражник доживают до 2005 года: к концу каждой эпохи у случайного 10 р.,
/// у арбитражника — 500 р.
fn traders() -> Report {
    let life = |strategy: Strategy, kopecks: i64| LifeOutcome {
        strategy,
        biography: BiographyId::new("student"),
        seed: 0,
        ending: None,
        last_day: date(2005, 1, 1),
        capital: Real::from_kopecks(kopecks),
        epoch_capitals: vec![Real::from_kopecks(kopecks); 4],
        events: [1985, 1990, 1995, 2000]
            .map(|year| (date(year, 3, 1), EventId::new("raikom_offer")))
            .to_vec(),
    };
    let outcomes = [
        life(Strategy::Random, 1_000),
        life(Strategy::Arbitrage, 50_000),
    ];
    Report::build(&content(), 1, &outcomes)
}

#[test]
fn an_arbitrage_edge_over_the_random_bot_outside_the_bounds_is_a_violation() {
    let scope = CheckScope::Epoch("stagnation".to_owned());
    let mut modest = rules();
    modest.trade.arbitrage_over_random = each_epoch(Bounds {
        from: 2.0,
        to: 10.0,
    });
    assert_eq!(
        check(&traders(), &trade(), &modest, &scope),
        Ok(vec![Violation::ArbitrageEdge {
            epoch: "stagnation".to_owned(),
            measured: 50.0,
            bounds: Bounds {
                from: 2.0,
                to: 10.0
            },
        }]),
        "перекупщик богаче в 50 раз"
    );
    assert_eq!(
        check(&report(), &trade(), &modest, &scope),
        Ok(Vec::new()),
        "без арбитражника перевес не меряется"
    );
}
