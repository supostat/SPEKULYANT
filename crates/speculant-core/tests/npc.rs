//! Соперники (§8): состав по сиду из пула шаблонов, торговля по правилам игрока и её след на
//! рынке, челноки между городами, доходы и судьбы, память обид и события с соперником.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{CityId, Content, EventId, GoodId, NpcId};
use speculant_core::economy::{Currency, Markets, Money, Real};
use speculant_core::npc::{Fate, Rivals, Strategy};
use speculant_core::rng::WorldRng;
use speculant_core::time::GameDate;
use speculant_core::{Command, Game, LifeSettings};

const NPC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/npc");
const DAY_MILLISECONDS: u32 = 1800;

fn load() -> Content {
    Content::load(Path::new(NPC)).expect("фикстура соперников валидна")
}

fn game_with(content: Content, seed: u64) -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается")
}

fn composition(game: &Game) -> BTreeSet<String> {
    game.rivals()
        .iter()
        .map(|rival| rival.npc().to_string())
        .collect()
}

/// Сид, в мире которого есть соперник с такой стратегией — среди первой тысячи сидов.
fn seed_with(strategy: Strategy) -> u64 {
    (1..=1000)
        .find(|seed| {
            game_with(load(), *seed)
                .rivals()
                .iter()
                .any(|rival| rival.strategy() == strategy)
        })
        .expect("стратегия встречается в одном из миров")
}

fn days(game: &mut Game, count: u32) {
    for _ in 0..count {
        assert!(game.advance(DAY_MILLISECONDS), "время идёт");
    }
}

#[test]
fn the_seed_picks_three_to_five_rivals_from_the_pool() {
    let content = load();
    let mut seen = BTreeSet::new();
    for seed in 1..=30 {
        let game = game_with(content.clone(), seed);
        let rivals = composition(&game);
        assert!((3..=5).contains(&rivals.len()), "сид {seed}: {rivals:?}");
        assert!(
            !rivals.contains("kofi"),
            "знакомый без стратегии — не соперник"
        );
        assert_eq!(rivals, composition(&game_with(content.clone(), seed)));
        for rival in game.rivals().iter() {
            assert!(["moscow", "leningrad"].contains(&rival.home().as_str()));
            if rival.strategy() == Strategy::Shuttle {
                assert!(
                    rival.partner_city().is_some(),
                    "у челнока есть второй город"
                );
            }
            assert_eq!(rival.fate(), Fate::Active);
        }
        seen.insert(rivals);
    }
    assert!(
        seen.len() >= 10,
        "состав зависит от сида: {} разных",
        seen.len()
    );
}

#[test]
fn trading_rivals_leave_their_mark_on_the_market() {
    let seed = seed_with(Strategy::Trader);
    let mut quiet = load();
    quiet.rival_rules.min_rivals = 0;
    quiet.rival_rules.max_rivals = 0;
    let mut alone = game_with(quiet, seed);
    let mut crowded = game_with(load(), seed);
    let stocks = |game: &Game| -> Vec<u32> {
        ["moscow", "leningrad"]
            .into_iter()
            .flat_map(|city| {
                ["jeans", "chewing_gum"]
                    .into_iter()
                    .map(move |good| (city, good))
            })
            .map(|(city, good)| {
                game.markets()
                    .city(&CityId::new(city))
                    .stock(&GoodId::new(good))
            })
            .collect()
    };
    let mut different_days = 0;
    for _ in 0..120 {
        days(&mut alone, 1);
        days(&mut crowded, 1);
        if stocks(&alone) != stocks(&crowded) {
            different_days += 1;
        }
    }
    assert!(
        different_days >= 30,
        "соперники двигают запасы: другой рынок в {different_days} днях из 120"
    );
    let trader = crowded
        .rivals()
        .iter()
        .find(|rival| rival.strategy() == Strategy::Trader)
        .expect("торговец в мире");
    let start = Money::new(Currency::Ruble, 50_000);
    assert_ne!(trader.trader().money().cash(), start, "торговец торговал");
}

#[test]
fn a_shuttle_carries_goods_to_the_dearer_city_and_comes_back_empty() {
    let seed = seed_with(Strategy::Shuttle);
    let mut game = game_with(load(), seed);
    let shuttle = game
        .rivals()
        .iter()
        .find(|rival| rival.strategy() == Strategy::Shuttle)
        .expect("челнок в мире")
        .npc()
        .clone();
    let mut loaded_trips = 0;
    let mut empty_trips = 0;
    for _ in 0..120 {
        days(&mut game, 1);
        let rival = game.rivals().get(&shuttle).expect("челнок есть");
        if rival.is_on_the_road() {
            if rival.trader().pocket().goods().next().is_some() {
                loaded_trips += 1;
            } else {
                empty_trips += 1;
            }
        }
    }
    assert!(loaded_trips >= 10, "с грузом в пути {loaded_trips} дней");
    assert!(empty_trips >= 10, "порожняком назад {empty_trips} дней");
}

#[test]
fn racketeers_collect_and_bankers_grow_rich() {
    let content = load();
    let mut rng = WorldRng::new(seed_with(Strategy::Banker));
    let mut markets = Markets::new(&content);
    let mut rivals = Rivals::new(&content, &mut rng, GameDate::START);
    let mut today = GameDate::START;
    for _ in 0..365 {
        today = today.next();
        rivals.next_day(&content, &mut markets, &mut rng, today);
    }
    for rival in rivals.iter() {
        let cash = rival.trader().money().cash().minor_units();
        match rival.strategy() {
            Strategy::Racketeer => assert_eq!(cash, 30_000 + 365 * 200, "дань 2 р. в день"),
            Strategy::Official => assert_eq!(cash, 100_000 + 365 * 100, "взятки 1 р. в день"),
            Strategy::Banker => assert!(
                (655_000..=657_000).contains(&cash),
                "30% годовых: одиннадцать первых чисел со 2 января, 5 000 р. × 1,025¹¹: {cash}"
            ),
            Strategy::Trader | Strategy::Shuttle => {}
        }
    }
}

#[test]
fn fates_take_rivals_out_and_the_zone_lets_them_back() {
    let calm = load();
    let mut doomed = load();
    for rules in doomed.rival_rules.strategies.values_mut() {
        rules.prison_chance = 1.0;
    }
    let mut rng = WorldRng::new(7);
    let mut markets = Markets::new(&calm);
    let mut rivals = Rivals::new(&calm, &mut rng, GameDate::START);
    let day_one = GameDate::START.next();
    rivals.next_day(&doomed, &mut markets, &mut rng, day_one);
    let released_on = day_one.after_days(100);
    for rival in rivals.iter() {
        assert_eq!(rival.fate(), Fate::Prison { until: released_on });
        assert_eq!(rival.fate().status(), "в зоне до 11.04.1984");
    }
    let mut today = day_one;
    while today < released_on {
        today = today.next();
        rivals.next_day(&calm, &mut markets, &mut rng, today);
    }
    assert!(
        rivals.iter().all(|rival| rival.fate() == Fate::Active),
        "срок вышел — снова в деле"
    );

    let mut fatal = load();
    for rules in fatal.rival_rules.strategies.values_mut() {
        rules.death_chance = 1.0;
    }
    rivals.next_day(&fatal, &mut markets, &mut rng, today.next());
    assert!(rivals.iter().all(|rival| rival.fate() == Fate::Dead));

    let mut poor = load();
    for template in poor.npcs.values_mut() {
        if template.capital.is_some() {
            template.capital = Some(Real::from_kopecks(200));
        }
    }
    let mut rng = WorldRng::new(7);
    let mut rivals = Rivals::new(&poor, &mut rng, GameDate::START);
    let mut markets = Markets::new(&poor);
    rivals.next_day(&poor, &mut markets, &mut rng, day_one);
    assert!(
        rivals
            .iter()
            .filter(|rival| rival.strategy() != Strategy::Racketeer)
            .all(|rival| rival.fate() == Fate::Ruined),
        "два рубля при пороге в пять — разорение"
    );
}

#[test]
fn a_rival_remembers_an_insult_and_comes_back_half_a_year_later() {
    let mut game = game_with(load(), seed_with(Strategy::Racketeer));
    game.start_event(&EventId::new("racket_visit"))
        .expect("рэкетир в мире");
    let rival = game
        .events()
        .pending()
        .and_then(|pending| pending.rival.clone())
        .expect("сид выбрал рэкетира");
    let name = game.content().npcs[&rival].name.clone();
    let shown = game.view().pending_event.expect("наезд ждёт ответа");
    assert_eq!(
        shown.text,
        format!("{name} подходит к твоему месту на толкучке: «Здесь платят».")
    );
    game.apply(Command::Choose { choice: 2 })
        .expect("послать можно всегда");
    let outcome = game
        .view()
        .pending_event
        .and_then(|event| event.outcome)
        .expect("исход");
    assert_eq!(outcome.text, format!("{name} обещает вернуться."));
    assert_eq!(outcome.effects, format!("{name} −2."));
    game.apply(Command::Continue).expect("исход прочитан");
    let memory = game.rivals().get(&rival).expect("рэкетир в мире").memory();
    assert_eq!(memory.relation(), -2);
    assert_eq!(memory.offended_days(game.today()), Some(0));
    assert_eq!(memory.history(), ["01.01.1984: Наезд"]);

    days(&mut game, 179);
    assert!(
        game.start_event(&EventId::new("old_grudge")).is_err(),
        "через 179 дней ещё рано"
    );
    days(&mut game, 1);
    game.start_event(&EventId::new("old_grudge"))
        .expect("через полгода обида возвращается");
    assert_eq!(
        game.events()
            .pending()
            .and_then(|pending| pending.rival.clone()),
        Some(rival),
        "вернулся тот, кого послали"
    );
}

#[test]
fn the_contacts_screen_lists_rivals_after_acquaintances() {
    let game = game_with(load(), 7);
    let contacts = game.view().contacts;
    assert_eq!(contacts[0].contact, "kofi");
    assert!(!contacts[0].rival);
    let rivals: Vec<_> = contacts.iter().skip(1).collect();
    assert_eq!(rivals.len(), game.rivals().len());
    for view in rivals {
        assert!(view.rival);
        assert_eq!(view.availability, "в деле");
        assert_eq!(view.relation, "знакомый");
        let template = &game.content().npcs[&NpcId::new(view.contact.as_str())];
        assert_eq!(view.role, template.role);
    }
}

#[test]
fn an_event_about_a_particular_rival_needs_him_in_this_world() {
    for seed in 1..=20 {
        let mut game = game_with(load(), seed);
        let present = game.rivals().get(&NpcId::new("banker_lev")).is_some();
        assert_eq!(
            game.start_event(&EventId::new("banker_offer")).is_ok(),
            present,
            "сид {seed}"
        );
    }
}
