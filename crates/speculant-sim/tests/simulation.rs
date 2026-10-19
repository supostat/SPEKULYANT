//! Прогон на настоящем ядре и данных: жизни раскладываются по стратегиям и открытым биографиям,
//! каждая доживает историю или кончается финалом, а итог не зависит от числа потоков.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use speculant_core::content::BiographyId;
use speculant_core::{Content, Game, LifeSettings};
use speculant_sim::{
    LifePlan, SimulationSettings, Strategy, horizon, plan_lives, play_life, simulate,
};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");

fn content() -> Arc<Content> {
    Arc::new(Content::load(Path::new(DATA)).expect("data/ валиден"))
}

fn settings(lives: usize, threads: usize) -> SimulationSettings {
    SimulationSettings {
        lives,
        seed: 1,
        strategies: Strategy::ALL.to_vec(),
        threads,
    }
}

#[test]
fn lives_go_round_the_strategies_and_the_biographies_open_from_the_start() {
    let content = content();
    let plans = plan_lives(&content, &settings(42, 1));
    let unlocked: Vec<&BiographyId> = content
        .biographies
        .iter()
        .filter(|(_, biography)| biography.starts_unlocked)
        .map(|(id, _)| id)
        .collect();
    assert_eq!(unlocked.len(), 5, "пять стартовых биографий (решение 15.5)");
    for (index, plan) in plans.iter().enumerate() {
        assert_eq!(plan.index, index);
        assert_eq!(plan.strategy, Strategy::ALL[index % 6]);
        assert_eq!(&plan.biography, unlocked[(index / 6) % 5]);
    }
    let seeds: BTreeSet<u64> = plans.iter().map(|plan| plan.seed).collect();
    assert_eq!(seeds.len(), plans.len(), "у каждой жизни свой сид");
    assert_eq!(
        plans,
        plan_lives(&content, &settings(42, 4)),
        "план — от сида прогона"
    );
}

#[test]
fn every_strategy_lives_the_history_through_and_threads_do_not_change_the_outcome() {
    let content = content();
    let alone = simulate(&content, &settings(6, 1));
    let together = simulate(&content, &settings(6, 3));
    assert_eq!(alone, together);
    let horizon = horizon(&content);
    for outcome in &alone {
        match &outcome.ending {
            None => {
                assert_eq!(
                    outcome.last_day, horizon,
                    "{}: дожил до песочницы",
                    outcome.strategy
                );
                assert_eq!(
                    outcome.epoch_capitals.len(),
                    4,
                    "капитал на конец четырёх эпох"
                );
            }
            Some(ending) => assert!(content.endings.contains_key(ending)),
        }
    }
    let strategies: Vec<Strategy> = alone.iter().map(|outcome| outcome.strategy).collect();
    assert_eq!(strategies, Strategy::ALL.to_vec());
}

#[test]
fn an_arbitrage_trader_ends_the_stagnation_richer_than_he_was_born() {
    let content = content();
    let plan = LifePlan {
        index: 0,
        strategy: Strategy::Arbitrage,
        biography: BiographyId::new("sailor"),
        seed: 11,
    };
    let born = Game::new_life(
        &LifeSettings {
            biography: "sailor".to_owned(),
            ironman: true,
            seed: plan.seed,
        },
        Arc::clone(&content),
    )
    .expect("моряк открыт с начала")
    .capital();
    let outcome = play_life(&content, &plan);
    let stagnation_end = outcome.epoch_capitals[0];
    assert!(
        stagnation_end.kopecks() > born.kopecks(),
        "к 1988-му {stagnation_end:?}, при рождении {born:?}"
    );
}
