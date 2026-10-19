//! Проверка покрытия: событие без срабатываний — ошибка, если оно не редкое; покрытие короче
//! порога или с других данных — тоже ошибка.

mod overlay;

use std::collections::BTreeMap;
use std::num::NonZeroUsize;

use speculant_core::content::EventId;
use speculant_sim::{Coverage, CoverageRules};
use speculant_validate::coverage;

const SOURCE: &str = "target/coverage.json";

fn rules(lives: usize) -> CoverageRules {
    CoverageRules {
        lives: NonZeroUsize::new(lives).expect("жизней больше нуля"),
        curiosity: 0.1,
    }
}

fn run(lives: usize, counts: &[(&str, usize)]) -> Coverage {
    Coverage {
        seed: 1,
        lives,
        saturated_at: 1,
        events: counts
            .iter()
            .map(|(event, count)| (EventId::new(*event), *count))
            .collect::<BTreeMap<_, _>>(),
    }
}

fn coverage_errors(overlays: &[&str], found: &Coverage, required_lives: usize) -> Vec<String> {
    let (content, files) = overlay::load(overlays);
    coverage::check(&content, &files, found, SOURCE, &rules(required_lives))
        .iter()
        .map(ToString::to_string)
        .collect()
}

const ALL_FIRED: [(&str, usize); 3] = [
    ("vodka_crate", 40),
    ("vodka_man_returns", 3),
    ("workshop_raid", 12),
];

#[test]
fn a_run_where_every_event_fired_is_clean() {
    assert_eq!(
        coverage_errors(&[], &run(600, &ALL_FIRED), 600),
        Vec::<String>::new()
    );
}

#[test]
fn an_event_that_never_fired_is_caught() {
    let found = run(
        600,
        &[
            ("vodka_crate", 40),
            ("vodka_man_returns", 0),
            ("workshop_raid", 12),
        ],
    );
    assert_eq!(
        coverage_errors(&[], &found, 600),
        [
            "events/stagnation/vodka_man_returns.toml:1: событие ни разу не сработало за 600 жизней прогона: поправь условия или пометь rare = true"
        ]
    );
}

#[test]
fn a_rare_event_may_never_fire() {
    let found = run(
        600,
        &[
            ("vodka_crate", 40),
            ("vodka_man_returns", 0),
            ("workshop_raid", 12),
        ],
    );
    assert_eq!(
        coverage_errors(&["rare_follow_up"], &found, 600),
        Vec::<String>::new()
    );
}

#[test]
fn a_run_shorter_than_the_threshold_is_caught() {
    assert_eq!(
        coverage_errors(&[], &run(200, &ALL_FIRED), 600),
        [
            "target/coverage.json: покрытие снято за 200 жизней, а [coverage] lives в balance.toml — 600"
        ]
    );
}

#[test]
fn coverage_of_other_data_is_caught() {
    let missing = run(600, &[("vodka_crate", 40), ("workshop_raid", 12)]);
    assert_eq!(
        coverage_errors(&[], &missing, 600),
        [
            "target/coverage.json: покрытие снято с других данных — в нём нет событий: vodka_man_returns; лишние: —"
        ]
    );
    let extra = run(
        600,
        &[
            ("vodka_crate", 40),
            ("vodka_man_returns", 3),
            ("old_event", 2),
            ("workshop_raid", 12),
        ],
    );
    assert_eq!(
        coverage_errors(&[], &extra, 600),
        [
            "target/coverage.json: покрытие снято с других данных — в нём нет событий: —; лишние: old_event"
        ]
    );
}

#[test]
fn the_summary_counts_fired_and_unseen_rare_events() {
    let (content, _) = overlay::load(&["rare_follow_up"]);
    let found = run(
        600,
        &[
            ("vodka_crate", 40),
            ("vodka_man_returns", 0),
            ("workshop_raid", 12),
        ],
    );
    assert_eq!(
        coverage::summary(&content, &found),
        "покрытие: за 600 жизней сработали 2 событий из 3, редких не встретилось 1"
    );
}
