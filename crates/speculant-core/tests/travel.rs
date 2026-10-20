//! Поездки (§4): направления из города по эпохам и по блату, стоимость и дни в пути, прибытие как
//! узел перемотки и дорожные события по весам маршрута.

use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{BiographyId, Content, EventId, NpcId};
use speculant_core::economy::Real;
use speculant_core::time::{FastForward, GameDate};
use speculant_core::view::{CityPriceView, DestinationView, JourneyView, MarketRowView};
use speculant_core::{Command, CommandError, Game, LifeSettings};

const TRAVEL: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/travel");
const DAY_MILLISECONDS: u32 = 1800;

fn content(daily_chance: f64, road_daily_chance: f64) -> Arc<Content> {
    let mut content = Content::load(Path::new(TRAVEL)).expect("фикстура поездок валидна");
    content.event_rules.daily_chance = daily_chance;
    content.event_rules.road_daily_chance = road_daily_chance;
    Arc::new(content)
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn game_with(content: Arc<Content>, seed: u64) -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, content).expect("жизнь начинается")
}

fn quiet_game() -> Game {
    game_with(content(0.0, 0.0), 7)
}

fn travel(destination: &str) -> Command {
    Command::Travel {
        destination: destination.to_owned(),
    }
}

/// Доводит игру до даты, отвечая на каждое событие первым выбором.
fn advance_to(game: &mut Game, target: GameDate) {
    while game.today() < target {
        game.advance(DAY_MILLISECONDS);
        answer_pending(game);
    }
}

fn answer_pending(game: &mut Game) {
    if game.events().pending().is_some() {
        game.apply(Command::Choose { choice: 1 })
            .expect("первый выбор доступен");
        game.apply(Command::Continue).expect("исход прочитан");
    }
}

/// Едет и ждёт прибытия, отвечая на события дороги.
fn ride(game: &mut Game, destination: &str) {
    game.apply(travel(destination)).expect("поездка возможна");
    while game.journey().is_some() {
        game.advance(DAY_MILLISECONDS);
        answer_pending(game);
    }
}

#[test]
fn the_city_lists_open_destinations_with_days_and_fare() {
    let game = quiet_game();
    assert_eq!(
        game.view().travel.destinations,
        vec![
            DestinationView {
                city: "leningrad".to_owned(),
                name: "Ленинград".to_owned(),
                days: 1,
                fare: "15 р.".to_owned(),
                by_connections: false,
                available: true,
                cheaper: Vec::new(),
                dearer: Vec::new(),
            },
            DestinationView {
                city: "berlin".to_owned(),
                name: "Берлин".to_owned(),
                days: 3,
                fare: "120 р.".to_owned(),
                by_connections: true,
                available: true,
                cheaper: vec!["одежда".to_owned()],
                dearer: vec!["алкоголь".to_owned()],
            },
        ],
        "Варшава до 1990 года закрыта и не показывается; одежда в Ленинграде дешевле на 5% — \
         разницу съедает разница цен покупки и продажи"
    );
    assert_eq!(game.view().travel.journey, None);
}

#[test]
fn a_trip_costs_its_fare_takes_its_days_and_ends_in_another_market() {
    let mut game = quiet_game();
    game.apply(travel("leningrad")).expect("в Ленинград можно");
    let view = game.view();
    assert_eq!(view.header.city, "В дороге");
    assert_eq!(view.header.money, vec!["Наличные: 985 р."]);
    assert_eq!(view.market.place, "В дороге: Москва — Ленинград");
    assert!(view.market.goods.is_empty(), "в поезде толкучки нет");
    assert_eq!(
        view.travel.journey,
        Some(JourneyView {
            from: "Москва".to_owned(),
            to: "Ленинград".to_owned(),
            arrives: "2 января 1984, пн".to_owned(),
            days_left: 1,
        })
    );
    assert_eq!(
        game.apply(Command::Buy {
            good: "jeans".to_owned(),
            quantity: 1
        }),
        Err(CommandError::Traveling)
    );
    assert_eq!(game.apply(travel("moscow")), Err(CommandError::Traveling));
    assert!(game.advance(DAY_MILLISECONDS));
    let view = game.view();
    assert_eq!(game.journey(), None);
    assert_eq!(view.header.city, "Ленинград");
    assert_eq!(view.market.place, "Галёрка у Гостиного двора");
    assert_eq!(
        view.market.goods[0].buy, "187",
        "джинсы здесь на 5% дешевле"
    );
    assert_eq!(
        view.messages,
        vec![
            "Выехал: Москва — Ленинград, 1 дн. за 15 р.",
            "Приехал: Ленинград"
        ]
    );
    let back: Vec<String> = view
        .travel
        .destinations
        .iter()
        .map(|destination| destination.name.clone())
        .collect();
    assert_eq!(back, vec!["Москва"]);
}

fn jeans_row(game: &Game) -> MarketRowView {
    game.view()
        .market
        .goods
        .into_iter()
        .find(|row| row.good == "jeans")
        .expect("джинсы на толкучке")
}

fn unknown(name: &str) -> CityPriceView {
    CityPriceView {
        name: name.to_owned(),
        here: false,
        seen: None,
        buy: String::new(),
        sell: String::new(),
        cheapest: false,
        dearest: false,
    }
}

#[test]
fn the_player_knows_the_prices_here_and_remembers_the_market_he_left() {
    let mut game = quiet_game();
    let born = jeans_row(&game);
    assert_eq!(
        born.cities,
        vec![
            CityPriceView {
                name: "Москва".to_owned(),
                here: true,
                seen: Some("01.01.1984".to_owned()),
                buy: born.buy.clone(),
                sell: born.sell.clone(),
                cheapest: false,
                dearest: false,
            },
            unknown("Берлин"),
            unknown("Ленинград"),
        ],
        "здесь — первым, остальные — по названию; сравнивать не с чем, Варшава закрыта"
    );
    game.apply(Command::Buy {
        good: "jeans".to_owned(),
        quantity: 2,
    })
    .expect("джинсы по карману");
    let left = jeans_row(&game);
    ride(&mut game, "leningrad");
    let arrived = jeans_row(&game);
    assert_eq!(
        arrived.cities,
        vec![
            CityPriceView {
                name: "Ленинград".to_owned(),
                here: true,
                seen: Some("02.01.1984".to_owned()),
                buy: arrived.buy.clone(),
                sell: arrived.sell.clone(),
                cheapest: true,
                dearest: false,
            },
            unknown("Берлин"),
            CityPriceView {
                name: "Москва".to_owned(),
                here: false,
                seen: Some("01.01.1984".to_owned()),
                buy: left.buy,
                sell: left.sell,
                cheapest: false,
                dearest: true,
            },
        ],
        "Москва — какой игрок оставил её после покупки: запас меньше, цены выше"
    );
}

#[test]
fn fast_forward_runs_until_the_arrival_and_stops_there() {
    let mut game = quiet_game();
    assert_eq!(
        game.apply(Command::FastForward {
            until: FastForward::UntilArrival
        }),
        Err(CommandError::NotTraveling)
    );
    game.apply(travel("berlin"))
        .expect("по блату через дядю Мишу");
    game.apply(Command::FastForward {
        until: FastForward::UntilArrival,
    })
    .expect("в пути можно перематывать");
    assert_eq!(
        game.view().time.fast_forward,
        Some(FastForward::UntilArrival)
    );
    assert!(game.advance(100_000), "сто секунд перемотки — дни идут");
    assert_eq!(game.today(), date(1984, 1, 4), "ровно три дня пути");
    assert_eq!(game.view().header.city, "Берлин");
    assert_eq!(
        game.fast_forward(),
        None,
        "прибытие останавливает перемотку"
    );
    assert!(
        !game.advance(1_000),
        "дальше — обычный ход: секунды на день мало"
    );
}

#[test]
fn travelling_by_connections_costs_a_favour_and_favours_run_out() {
    let mut game = quiet_game();
    ride(&mut game, "berlin");
    let misha = &game.contacts()[&NpcId::new("uncle_misha")];
    assert_eq!(misha.debt(), -1);
    assert_eq!(misha.history(), ["01.01.1984: помог с выездом: Берлин"]);
    ride(&mut game, "moscow");
    assert_eq!(
        game.apply(travel("berlin")),
        Err(CommandError::NeedsConnections {
            city: "berlin".to_owned(),
            name: "Берлин".to_owned()
        }),
        "одну услугу дядя Миша терпит, вторую — нет"
    );
    let berlin = game
        .view()
        .travel
        .destinations
        .into_iter()
        .find(|destination| destination.city == "berlin")
        .expect("Берлин в списке");
    assert!(!berlin.available, "по блату помочь некому");
    advance_to(&mut game, date(1990, 1, 1));
    let berlin = game
        .view()
        .travel
        .destinations
        .into_iter()
        .find(|destination| destination.city == "berlin")
        .expect("Берлин в списке");
    assert!(
        !berlin.by_connections && berlin.available,
        "с 1990 года — без блата"
    );
    game.apply(travel("berlin")).expect("Берлин открыт");
}

#[test]
fn a_closed_city_opens_with_its_date() {
    let mut game = quiet_game();
    advance_to(&mut game, date(1990, 5, 31));
    assert_eq!(
        game.apply(travel("warsaw")),
        Err(CommandError::CityClosed {
            city: "warsaw".to_owned(),
            name: "Варшава".to_owned()
        })
    );
    advance_to(&mut game, date(1990, 6, 1));
    assert!(
        game.view()
            .travel
            .destinations
            .iter()
            .any(|destination| destination.city == "warsaw")
    );
    ride(&mut game, "warsaw");
    assert_eq!(game.view().market.place, "Стадион Десятилетия");
}

#[test]
fn unknown_cities_missing_routes_and_empty_pockets_are_refused() {
    let mut game = quiet_game();
    assert_eq!(
        game.apply(travel("paris")),
        Err(CommandError::UnknownCity {
            city: "paris".to_owned()
        })
    );
    assert_eq!(
        game.apply(travel("moscow")),
        Err(CommandError::NoRoute {
            from: "moscow".to_owned(),
            to: "moscow".to_owned()
        })
    );
    game.apply(Command::Deposit {
        place: "mattress".to_owned(),
        amount: 990,
    })
    .expect("под матрас");
    assert_eq!(
        game.apply(travel("leningrad")),
        Err(CommandError::NotEnoughCash {
            needed: "15 р.".to_owned(),
            cash: "10 р.".to_owned()
        })
    );
    assert!(
        game.view()
            .travel
            .destinations
            .iter()
            .all(|destination| !destination.available)
    );
    assert_eq!(
        game.contacts()[&NpcId::new("uncle_misha")].debt(),
        0,
        "без денег на билет услугу не просят"
    );
}

#[test]
fn road_events_happen_only_on_the_road_by_the_weights_of_the_route() {
    let mut eventful = Arc::unwrap_or_clone(content(1.0, 1.0));
    eventful
        .biographies
        .get_mut(&BiographyId::new("student"))
        .expect("биография студента есть")
        .start_cash = Real::from_kopecks(1_000_000);
    let mut game = game_with(Arc::new(eventful), 7);
    advance_to(&mut game, date(1990, 6, 1));
    let mut seen_on_road = Vec::new();
    let mut seen_at_home = Vec::new();
    for _ in 0..40 {
        for destination in ["warsaw", "moscow"] {
            game.apply(travel(destination)).expect("Варшава открыта");
            while game.journey().is_some() {
                game.advance(DAY_MILLISECONDS);
                if let Some(pending) = game.events().pending() {
                    let seen = if game.journey().is_some() {
                        &mut seen_on_road
                    } else {
                        &mut seen_at_home
                    };
                    seen.push(pending.event.clone());
                }
                answer_pending(&mut game);
            }
            game.advance(DAY_MILLISECONDS);
            if let Some(pending) = game.events().pending() {
                seen_at_home.push(pending.event.clone());
            }
            answer_pending(&mut game);
        }
    }
    let count =
        |events: &[EventId], id: &str| events.iter().filter(|event| event.as_str() == id).count();
    let (customs, robbery) = (
        count(&seen_on_road, "customs"),
        count(&seen_on_road, "robbery"),
    );
    assert_eq!(
        customs + robbery,
        seen_on_road.len(),
        "в дороге — только дорожные события маршрута: {seen_on_road:?}"
    );
    assert!(
        customs >= 2 * robbery && customs <= 5 * robbery,
        "веса 3 и 1: таможня {customs}, гость {robbery}"
    );
    assert!(!seen_at_home.is_empty());
    assert!(
        seen_at_home
            .iter()
            .all(|event| event.as_str() == "home_rain"),
        "дома — только домашние события: {seen_at_home:?}"
    );
}

#[test]
fn an_event_on_the_road_stops_fast_forward() {
    let mut game = game_with(content(0.0, 1.0), 7);
    advance_to(&mut game, date(1990, 6, 1));
    game.apply(travel("warsaw")).expect("Варшава открыта");
    game.apply(Command::FastForward {
        until: FastForward::UntilArrival,
    })
    .expect("в пути можно перематывать");
    game.advance(100_000);
    assert_eq!(game.today(), date(1990, 6, 2), "первый же день пути");
    assert!(game.events().pending().is_some());
    assert!(game.journey().is_some(), "ещё в пути");
    assert_eq!(game.fast_forward(), None);
}

#[test]
fn the_same_seed_gives_the_same_trips() {
    let journal_of = |seed: u64| -> Vec<String> {
        let mut game = game_with(content(0.5, 0.5), seed);
        advance_to(&mut game, date(1990, 6, 1));
        for _ in 0..10 {
            ride(&mut game, "warsaw");
            ride(&mut game, "moscow");
        }
        game.journal()
            .iter()
            .map(|entry| format!("{} {}", entry.date.to_iso(), entry.event))
            .collect()
    };
    assert_eq!(journal_of(7), journal_of(7));
    assert_ne!(journal_of(7), journal_of(8));
}

#[test]
fn an_event_on_the_last_day_of_the_road_delays_the_arrival_until_it_is_read() {
    let mut game = game_with(content(0.0, 1.0), 7);
    game.apply(travel("leningrad")).expect("в Ленинград можно");
    assert!(game.advance(DAY_MILLISECONDS));
    let shown = game.view().pending_event.expect("проводник пришёл");
    assert_eq!(shown.title, "Проводник");
    assert_eq!(game.view().header.city, "В дороге", "событие ещё в поезде");
    game.apply(Command::Choose { choice: 1 })
        .expect("выбор доступен");
    game.apply(Command::Continue).expect("исход прочитан");
    assert_eq!(game.journey(), None);
    assert_eq!(game.view().header.city, "Ленинград");
    assert_eq!(game.today(), date(1984, 1, 2), "приезд в тот же день");
}
