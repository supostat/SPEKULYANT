//! Соперники (§8): состав по сиду из пула шаблонов, торговля по правилам игрока и её след на
//! рынке, челноки между городами по сегодняшним ценам и их сделки в ленте игрока, доходы и судьбы,
//! память обид и события с соперником.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{CityId, Content, EventId, GoodId, NpcId, RivalCondition};
use speculant_core::economy::{Currency, Markets, Money, Real};
use speculant_core::npc::{DealSide, Fate, FateKind, Rival, Rivals, Strategy};
use speculant_core::rng::WorldRng;
use speculant_core::time::GameDate;
use speculant_core::{Command, Game, LifeSettings};

const NPC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/npc");
const DAY_MILLISECONDS: u32 = 1800;
/// Дрейф цен в фикстуре выключен: сид дрейфа рынков ни на что не влияет.
const DRIFT_SEED: u64 = 1;

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

/// Челнок Толик из Москвы в Ленинград с деньгами на любой лот.
fn shuttle_from_moscow(content: &Content) -> Rival {
    Rival::new(
        NpcId::new("shuttle_tolik"),
        Strategy::Shuttle,
        Money::new(Currency::Ruble, 100_000),
        CityId::new("moscow"),
        Some(CityId::new("leningrad")),
        content,
    )
}

#[test]
fn a_shuttle_picks_its_cargo_by_todays_prices_of_both_markets() {
    let content = load();
    let today = GameDate::START;
    let mut rng = WorldRng::new(7);
    let mut cargo = |markets: &mut Markets| -> Vec<(String, DealSide)> {
        shuttle_from_moscow(&content)
            .live_day(&content, markets, rng.stream("npc"), today)
            .into_iter()
            .map(|deal| (deal.good.to_string(), deal.side))
            .collect()
    };
    let mut usual = Markets::new(&content, DRIFT_SEED);
    assert_eq!(
        cargo(&mut usual),
        [("jeans".to_owned(), DealSide::Bought)],
        "джинсы в Ленинграде дороже в 1,3 раза"
    );
    let mut glutted = Markets::new(&content, DRIFT_SEED);
    glutted.apply_shock(
        GoodId::new("chewing_gum"),
        Some(CityId::new("moscow")),
        0.3,
        Some(30),
        today,
    );
    assert_eq!(
        cargo(&mut glutted),
        [("chewing_gum".to_owned(), DealSide::Bought)],
        "завал жвачки в Москве — сегодня она выгоднее джинсов"
    );
    let mut flooded = Markets::new(&content, DRIFT_SEED);
    flooded.put(&CityId::new("leningrad"), &GoodId::new("jeans"), 60);
    assert_eq!(
        cargo(&mut flooded),
        [],
        "Ленинград завален джинсами — по сегодняшним ценам везти нечего"
    );
}

#[test]
fn a_shuttle_deal_on_the_players_market_is_a_line_in_the_feed() {
    let seed = seed_with(Strategy::Shuttle);
    let mut game = game_with(load(), seed);
    let shuttles: Vec<String> = game
        .rivals()
        .iter()
        .filter(|rival| rival.strategy() == Strategy::Shuttle)
        .map(|rival| game.content().npcs[rival.npc()].name.clone())
        .collect();
    let mut deals = BTreeSet::new();
    for _ in 0..60 {
        days(&mut game, 1);
        deals.extend(
            game.view()
                .messages
                .into_iter()
                .filter(|line| line.starts_with("Скупка: ") || line.starts_with("Привоз: ")),
        );
    }
    assert!(
        deals.iter().any(|line| shuttles
            .iter()
            .any(|name| line.ends_with(&format!("— {name}")))),
        "сделки челноков {shuttles:?} в Москве видны в ленте: {deals:?}"
    );
}

#[test]
fn racketeers_collect_and_bankers_grow_rich() {
    let content = load();
    let mut rng = WorldRng::new(seed_with(Strategy::Banker));
    let mut markets = Markets::new(&content, DRIFT_SEED);
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
    let mut markets = Markets::new(&calm, DRIFT_SEED);
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
    let mut markets = Markets::new(&poor, DRIFT_SEED);
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

/// Соперники мира, чья судьба подходит под отбор, — набором.
fn rivals_where(game: &Game, fits: impl Fn(Fate) -> bool) -> BTreeSet<NpcId> {
    game.rivals()
        .iter()
        .filter(|rival| fits(rival.fate()))
        .map(|rival| rival.npc().clone())
        .collect()
}

#[test]
fn an_event_about_a_deputy_picks_a_deputy_and_others_pick_rivals_in_business() {
    let mut content = load();
    content
        .rival_rules
        .strategies
        .get_mut(&Strategy::Official)
        .expect("чиновник в правилах фикстуры")
        .deputy_chance = 1.0;
    let mut game = game_with(content, seed_with(Strategy::Official));
    let deputy_call = EventId::new("deputy_call");
    assert!(
        game.start_event(&deputy_call).is_err(),
        "пока депутатов нет, звонить некому"
    );
    days(&mut game, 1);
    let deputies = rivals_where(&game, |fate| fate == Fate::Deputy);
    assert!(!deputies.is_empty(), "чиновник ушёл в депутаты");
    let in_business = rivals_where(&game, Fate::is_active);
    let matching = |condition: RivalCondition| -> BTreeSet<NpcId> {
        game.rivals()
            .matching(&condition, game.today())
            .into_iter()
            .cloned()
            .collect()
    };
    assert_eq!(
        matching(RivalCondition {
            fate: FateKind::Deputy,
            ..RivalCondition::default()
        }),
        deputies
    );
    assert_eq!(
        matching(RivalCondition::default()),
        in_business,
        "без судьбы в условии — соперники в деле"
    );
    game.start_event(&deputy_call).expect("депутат в мире");
    let caller = game
        .events()
        .pending()
        .and_then(|pending| pending.rival.clone())
        .expect("сид выбрал соперника");
    assert!(deputies.contains(&caller), "звонит депутат");
}
