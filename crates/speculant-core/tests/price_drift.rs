//! Дрейф цен и сезоны (§5.2 п. 6): у товара каждый месяц своё отклонение от кривой по сиду мира, у
//! категории — сезон, а от середины месяца к середине следующего цена идёт по прямой.

use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{CategoryId, CityId, Content, GoodId};
use speculant_core::economy::Markets;
use speculant_core::time::GameDate;
use speculant_core::{Game, LifeSettings};

const ECONOMY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/economy");
const AMPLITUDE: f64 = 0.3;
/// Два года по дням — два десятка стыков месяцев.
const DAYS: u32 = 731;
/// В самом коротком месяце 28 дней: быстрее цена от опоры к опоре не идёт.
const SHORTEST_MONTH_DAYS: f64 = 28.0;
const JUNE: usize = 5;

/// Фикстура экономики с дрейфом: шоков в ней нет, сезонов у категорий тоже.
fn drifting(amplitude: f64) -> Content {
    let mut content = Content::load(Path::new(ECONOMY)).expect("фикстура экономики валидна");
    content.drift.amplitude = amplitude;
    content
}

fn every_day() -> impl Iterator<Item = GameDate> {
    (0..DAYS).map(|day| GameDate::START.after_days(day))
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата есть в календаре")
}

/// Во сколько раз цена товара в Москве ушла от кривой с множителем города: дрейф и сезон.
fn drift_of(content: &Content, markets: &Markets, good: &str, date: GameDate) -> f64 {
    let moscow = CityId::new("moscow");
    let good = GoodId::new(good);
    let item = &content.goods[&good];
    let curve = item.curve.kopecks_on(date).expect("товар продаётся");
    markets
        .normal_kopecks(content, &moscow, &good, date)
        .expect("товар продаётся")
        / (curve * content.cities[&moscow].multiplier(&item.category))
}

#[test]
fn neighbouring_months_are_stitched_without_a_jump() {
    let content = drifting(AMPLITUDE);
    let markets = Markets::new(&content, 7);
    let drift: Vec<f64> = every_day()
        .map(|date| drift_of(&content, &markets, "jeans", date))
        .collect();
    let largest_step = 2.0 * AMPLITUDE / SHORTEST_MONTH_DAYS;
    for (day, pair) in drift.windows(2).enumerate() {
        assert!(
            (pair[1] - pair[0]).abs() <= largest_step + 1e-12,
            "день {day}: {} → {} — скачок",
            pair[0],
            pair[1]
        );
    }
    let lowest = drift.iter().copied().fold(f64::INFINITY, f64::min);
    let highest = drift.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    assert!(
        1.0 - AMPLITUDE <= lowest && highest <= 1.0 + AMPLITUDE,
        "дрейф {lowest}–{highest} не шире размаха"
    );
    assert!(
        highest - lowest > AMPLITUDE,
        "за два года цена заметно ходит: {lowest}–{highest}"
    );
    let video_recorder: Vec<f64> = every_day()
        .map(|date| drift_of(&content, &markets, "video_recorder", date))
        .collect();
    assert_ne!(video_recorder, drift, "у каждого товара свой дрейф");
}

#[test]
fn a_category_season_moves_its_goods_and_leaves_the_others() {
    let mut content = drifting(0.0);
    let mut season = [1.0; 12];
    season[JUNE] = 1.5;
    content
        .categories
        .get_mut(&CategoryId::new("clothes"))
        .expect("одежда в фикстуре")
        .season = Some(season);
    let markets = Markets::new(&content, 7);
    let jeans = |date: GameDate| drift_of(&content, &markets, "jeans", date);
    assert!(
        (jeans(date(1985, 6, 16)) - 1.5).abs() < 1e-9,
        "середина июня — ровно июньский множитель"
    );
    let first_of_july = jeans(date(1985, 7, 1));
    assert!(
        1.0 < first_of_july && first_of_july < 1.5,
        "к середине июля цена спускается по прямой: {first_of_july}"
    );
    assert!(
        (jeans(date(1985, 3, 16)) - 1.0).abs() < 1e-9,
        "весной сезон одежды цену не трогает"
    );
    assert!(
        (drift_of(&content, &markets, "vodka_bottle", date(1985, 6, 16)) - 1.0).abs() < 1e-9,
        "у алкоголя сезона нет"
    );
}

#[test]
fn a_life_drifts_by_its_world_seed() {
    let content = Arc::new(drifting(AMPLITUDE));
    let drift_of_life = |seed: u64| -> Vec<f64> {
        let settings = LifeSettings {
            biography: "student".to_owned(),
            ironman: true,
            seed,
        };
        let game = Game::new_life(&settings, Arc::clone(&content)).expect("жизнь начинается");
        every_day()
            .map(|date| drift_of(&content, game.markets(), "jeans", date))
            .collect()
    };
    let by_seed = every_day()
        .map(|date| drift_of(&content, &Markets::new(&content, 7), "jeans", date))
        .collect::<Vec<f64>>();
    assert_eq!(drift_of_life(7), by_seed, "дрейф жизни — по сиду её мира");
    assert_eq!(drift_of_life(7), drift_of_life(7));
    assert_ne!(
        drift_of_life(7),
        drift_of_life(8),
        "другой сид — другой дрейф"
    );
}
