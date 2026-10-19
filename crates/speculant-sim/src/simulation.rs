//! Прогон: жизни раскладываются по стратегиям и биографиям, проживаются параллельно, а итог не
//! зависит от числа потоков — каждая жизнь знает свой номер и свой сид.

use std::panic::{self, AssertUnwindSafe};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use rand_chacha::rand_core::Rng;
use speculant_core::Content;
use speculant_core::content::BiographyId;
use speculant_core::rng::WorldRng;

use crate::life_run::play_life;
use crate::{LifeOutcome, LifePlan, Strategy};

/// Поток, из которого прогон тянет сиды жизней.
const LIVES_STREAM: &str = "lives";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimulationSettings {
    pub lives: usize,
    pub seed: u64,
    /// Стратегии по кругу: жизнь `n` играет стратегией `n` по модулю их числа.
    pub strategies: Vec<Strategy>,
    pub threads: usize,
}

/// Планы жизней: стратегии по кругу, биографии, открытые с начала, — по кругу внутри каждой
/// стратегии, сиды — из потока прогона.
pub fn plan_lives(content: &Content, settings: &SimulationSettings) -> Vec<LifePlan> {
    assert!(
        !settings.strategies.is_empty(),
        "прогон играет хотя бы одной стратегией"
    );
    let biographies: Vec<&BiographyId> = content
        .biographies
        .iter()
        .filter(|(_, biography)| biography.starts_unlocked)
        .map(|(id, _)| id)
        .collect();
    assert!(
        !biographies.is_empty(),
        "хотя бы одна биография открыта с начала"
    );
    let mut seeds = WorldRng::new(settings.seed);
    let strategies = settings.strategies.len();
    (0..settings.lives)
        .map(|index| LifePlan {
            index,
            strategy: settings.strategies[index % strategies],
            biography: biographies[(index / strategies) % biographies.len()].clone(),
            seed: seeds.stream(LIVES_STREAM).next_u64(),
        })
        .collect()
}

/// Проживает все жизни прогона в `threads` потоках; исходы — в порядке планов.
pub fn simulate(content: &Arc<Content>, settings: &SimulationSettings) -> Vec<LifeOutcome> {
    assert!(settings.threads > 0, "прогон идёт хотя бы в одном потоке");
    let plans = plan_lives(content, settings);
    let next = AtomicUsize::new(0);
    let outcomes = Mutex::new(vec![None; plans.len()]);
    thread::scope(|scope| {
        for _ in 0..settings.threads.min(plans.len()) {
            scope.spawn(|| {
                while let Some(plan) = plans.get(next.fetch_add(1, Ordering::Relaxed)) {
                    let outcome = live_or_tell(content, plan);
                    outcomes.lock().expect("поток не падал с замком")[plan.index] = Some(outcome);
                }
            });
        }
    });
    outcomes
        .into_inner()
        .expect("потоки закончились")
        .into_iter()
        .map(|outcome| outcome.expect("каждую жизнь прожил какой-то поток"))
        .collect()
}

/// Жизнь, а если ядро или бот упали — сначала называет её, чтобы её можно было повторить.
fn live_or_tell(content: &Arc<Content>, plan: &LifePlan) -> LifeOutcome {
    panic::catch_unwind(AssertUnwindSafe(|| play_life(content, plan))).unwrap_or_else(|failure| {
        eprintln!(
            "жизнь {} упала: стратегия {}, биография {}, сид жизни {}",
            plan.index, plan.strategy, plan.biography, plan.seed
        );
        panic::resume_unwind(failure)
    })
}
