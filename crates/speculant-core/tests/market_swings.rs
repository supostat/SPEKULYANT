//! Дефициты и завалы толкучек (§5.2 п. 4): рынок сам по сиду мира рождает шоки «город × товар»,
//! в городе игрока о них говорит лента, о других городах — слухи, и часть слухов путает город.

use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{CityId, Content};
use speculant_core::economy::{Markets, Shock};
use speculant_core::{Command, Game, LifeSettings};

const ECONOMY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/economy");
const DAY_MILLISECONDS: u32 = 1800;

/// Фикстура экономики, где дефицит или завал рождается каждый день, без событий дня.
fn swinging(deficit_share: f64, rumor_accuracy: f64) -> Content {
    let mut content = Content::load(Path::new(ECONOMY)).expect("фикстура экономики валидна");
    content.swings.daily_chance = 1.0;
    content.swings.deficit_share = deficit_share;
    content.swings.rumor_accuracy = rumor_accuracy;
    content.event_rules.daily_chance = 0.0;
    content
}

fn game_with(content: Content, seed: u64) -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается")
}

fn live(game: &mut Game, days: u32) {
    for _ in 0..days {
        assert!(game.advance(DAY_MILLISECONDS), "день проходит");
    }
}

/// Шоки, привязанные к городу и действующие сегодня.
fn local_shocks(game: &Game) -> Vec<Shock> {
    game.markets()
        .shocks()
        .iter()
        .filter(|shock| shock.city.is_some() && !shock.is_over_on(game.today()))
        .cloned()
        .collect()
}

#[test]
fn every_day_by_the_seed_a_deficit_or_a_glut_lands_on_a_market_within_the_rules() {
    let mut game = game_with(swinging(0.5, 1.0), 7);
    let rules = game.content().swings;
    live(&mut game, 30);
    let shocks = local_shocks(&game);
    assert!(shocks.len() >= 2, "за месяц родились шоки: {shocks:?}");
    assert!(
        shocks.iter().any(|shock| shock.factor > 1.0),
        "есть дефицит"
    );
    assert!(shocks.iter().any(|shock| shock.factor < 1.0), "есть завал");
    for shock in &shocks {
        let factor = shock.factor;
        let in_deficit = rules.deficit_factor.from <= factor && factor <= rules.deficit_factor.to;
        let in_glut = rules.glut_factor.from <= factor && factor <= rules.glut_factor.to;
        assert!(
            in_deficit || in_glut,
            "множитель {factor} в пределах правил"
        );
        let until = shock.until.expect("дефицит и завал кончаются");
        let days = until.day_index() - game.today().day_index();
        assert!(
            days <= i32::try_from(rules.days.to).expect("срок мал"),
            "срок не длиннее правил"
        );
        assert!(Markets::is_on_sale(
            game.content(),
            &shock.good,
            game.today()
        ));
    }
}

#[test]
fn a_good_in_a_city_carries_one_swing_at_a_time() {
    let mut game = game_with(swinging(0.5, 1.0), 7);
    live(&mut game, 120);
    let shocks = local_shocks(&game);
    for (index, shock) in shocks.iter().enumerate() {
        assert!(
            !shocks[index + 1..]
                .iter()
                .any(|other| other.city == shock.city && other.good == shock.good),
            "два шока на {} в {:?} не множатся",
            shock.good,
            shock.city
        );
    }
}

#[test]
fn the_same_seed_gives_the_same_swings_and_another_seed_others() {
    let swings_of = |seed: u64| {
        let mut game = game_with(swinging(0.5, 1.0), seed);
        live(&mut game, 60);
        local_shocks(&game)
    };
    assert_eq!(swings_of(7), swings_of(7));
    assert_ne!(swings_of(7), swings_of(8));
}

#[test]
fn a_deficit_raises_the_price_only_in_its_city_and_a_glut_lowers_it() {
    for (deficit_share, rises) in [(1.0, true), (0.0, false)] {
        let mut game = game_with(swinging(deficit_share, 1.0), 7);
        live(&mut game, 1);
        let shock = local_shocks(&game)
            .pop()
            .expect("шок родился в первый же день");
        let city = shock.city.clone().expect("шок с городом");
        let other = game
            .content()
            .cities
            .keys()
            .find(|candidate| **candidate != city)
            .expect("в фикстуре два города")
            .clone();
        let content = game.content();
        let ratio = |city: &CityId| {
            let city_rules = &content.cities[city];
            let good = &content.goods[&shock.good];
            let curve = good
                .curve
                .kopecks_on(game.today())
                .expect("товар продаётся");
            game.markets()
                .normal_kopecks(content, city, &shock.good, game.today())
                .expect("товар продаётся")
                / (curve * city_rules.multiplier(&good.category))
        };
        assert!(
            (ratio(&city) - shock.factor).abs() < 1e-9,
            "цена — в {}",
            shock.factor
        );
        assert!(
            (ratio(&other) - 1.0).abs() < 1e-9,
            "в другом городе цена своя"
        );
        assert_eq!(shock.factor > 1.0, rises);
    }
}

#[test]
fn a_swing_in_the_players_city_is_news_in_the_feed() {
    let mut game = game_with(swinging(0.5, 1.0), 7);
    let here = game.city().clone();
    let mut news = None;
    for _ in 0..60 {
        live(&mut game, 1);
        let born_here = local_shocks(&game)
            .into_iter()
            .find(|shock| shock.city.as_ref() == Some(&here));
        if let Some(shock) = born_here {
            news = Some(shock);
            break;
        }
    }
    let shock = news.expect("за два месяца дефицит или завал случился и здесь");
    let name = &game.content().goods[&shock.good].name;
    let label = if shock.factor > 1.0 {
        "На толкучке дефицит"
    } else {
        "На толкучке завал"
    };
    assert!(
        game.view().messages.contains(&format!("{label}: {name}")),
        "{:?}",
        game.view().messages
    );
}

#[test]
fn rumors_name_the_freshest_swings_of_other_cities_and_never_the_city_of_the_player() {
    let mut game = game_with(swinging(0.5, 1.0), 7);
    live(&mut game, 60);
    let here = &game.content().cities[game.city()].name;
    let rumors = game.view().rumors;
    let rules = game.content().swings;
    assert!(
        !rumors.is_empty() && rumors.len() <= rules.rumors,
        "{rumors:?}"
    );
    for rumor in &rumors {
        assert!(
            !rumor.starts_with(here.as_str()),
            "о своём городе игрок знает сам: {rumor}"
        );
        assert!(
            rumor.contains(": дефицит — ") || rumor.contains(": завал — "),
            "{rumor}"
        );
    }
}

#[test]
fn an_inaccurate_rumor_names_another_city_and_stays_the_same_from_frame_to_frame() {
    let mut accurate = game_with(swinging(0.5, 1.0), 7);
    let mut inaccurate = game_with(swinging(0.5, 0.0), 7);
    live(&mut accurate, 60);
    live(&mut inaccurate, 60);
    let truth = accurate.view().rumors;
    let told = inaccurate.view().rumors;
    assert_eq!(truth.len(), told.len());
    for (true_rumor, told_rumor) in truth.iter().zip(&told) {
        let city = |rumor: &str| rumor.split(':').next().expect("город в слухе").to_owned();
        assert_ne!(city(true_rumor), city(told_rumor), "слух путает город");
    }
    assert_eq!(
        inaccurate.view().rumors,
        told,
        "снимок не перебрасывает кости"
    );
    inaccurate
        .apply(Command::Pause { paused: true })
        .expect("пауза ставится");
    assert_eq!(inaccurate.view().rumors, told);
}
