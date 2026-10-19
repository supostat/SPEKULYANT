//! Контракт моста: снимок мира и команды переживают JSON без потерь, фикстура рынка — валидный снимок.

use std::path::Path;
use std::sync::Arc;

use speculant_core::time::FastForward;
use speculant_core::view::Screen;
use speculant_core::{Command, CommandError, Content, Game, LifeSettings, StateView};

const MARKET_FIXTURE: &str = include_str!("../../../design/etalons/market.view.json");
/// Фикстуры макетов дизайн-книги и экраны, которые они показывают.
const ETALON_FIXTURES: [(&str, Screen); 5] = [
    (MARKET_FIXTURE, Screen::Market),
    (
        include_str!("../../../design/etalons/event.view.json"),
        Screen::Event,
    ),
    (
        include_str!("../../../design/etalons/manager.view.json"),
        Screen::Manager,
    ),
    (
        include_str!("../../../design/etalons/pager.view.json"),
        Screen::Pager,
    ),
    (
        include_str!("../../../design/etalons/end.view.json"),
        Screen::Summary,
    ),
];
const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");

fn life_settings() -> LifeSettings {
    LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed: 7,
    }
}

fn content() -> Arc<Content> {
    Arc::new(Content::load(Path::new(DATA)).expect("data/ валиден"))
}

fn new_life() -> Game {
    Game::new_life(&life_settings(), content()).expect("жизнь начинается")
}

#[test]
fn market_fixture_is_a_state_view_of_the_market_mockup() {
    let view: StateView =
        serde_json::from_str(MARKET_FIXTURE).expect("фикстура рынка разбирается в StateView");
    assert_eq!(view.screen, Screen::Market);
    assert_eq!(view.header.city, "Москва");
    assert_eq!(view.market.goods.len(), 8);
    assert_eq!(view.market.goods[4].buy, "4.50");
}

#[test]
fn every_etalon_fixture_is_a_state_view_of_its_screen() {
    for (fixture, screen) in ETALON_FIXTURES {
        let view: StateView = serde_json::from_str(fixture)
            .unwrap_or_else(|error| panic!("фикстура {screen:?} не разбирается: {error}"));
        assert_eq!(view.screen, screen);
    }
}

#[test]
fn state_view_survives_a_json_round_trip() {
    let view: StateView = serde_json::from_str(MARKET_FIXTURE).expect("фикстура разбирается");
    let json = serde_json::to_string(&view).expect("снимок сериализуется");
    let again: StateView = serde_json::from_str(&json).expect("снимок разбирается обратно");
    assert_eq!(again, view);
}

#[test]
fn every_command_survives_a_json_round_trip_under_its_own_name() {
    let commands = vec![
        Command::NewLife(life_settings()),
        Command::Advance { milliseconds: 1800 },
        Command::SetSpeed { speed: 2 },
        Command::Pause { paused: true },
        Command::FastForward {
            until: FastForward::UntilArrival,
        },
        Command::Buy {
            good: "jeans".to_owned(),
            quantity: 3,
        },
        Command::Sell {
            good: "jeans".to_owned(),
            quantity: 1,
        },
        Command::Travel {
            destination: "leningrad".to_owned(),
        },
        Command::Choose { choice: 2 },
        Command::Continue,
        Command::OpenScreen {
            screen: Screen::Pager,
        },
        Command::OpenBusiness {
            kind: "kiosks".to_owned(),
        },
        Command::SetPolicy {
            business: "kiosks@moscow".to_owned(),
            policy: "markup".to_owned(),
            option: 1,
        },
        Command::Hire {
            business: "kiosks@moscow".to_owned(),
            candidate: "gena_kabanov".to_owned(),
        },
        Command::Fire {
            business: "kiosks@moscow".to_owned(),
            employee: "gena_kabanov".to_owned(),
        },
        Command::Call {
            contact: "lena".to_owned(),
        },
        Command::ErasePagerMessage { message: 2 },
        Command::Deposit {
            place: "savings_bank".to_owned(),
            amount: 500,
        },
        Command::Withdraw {
            place: "savings_bank".to_owned(),
            amount: 100,
        },
        Command::Save {
            slot: "ironman".to_owned(),
        },
        Command::Load {
            slot: "ironman".to_owned(),
        },
        Command::Quit,
    ];
    for command in commands {
        let json = serde_json::to_value(&command).expect("команда сериализуется");
        assert_eq!(json["command"], command.name(), "имя команды в JSON");
        let again: Command = serde_json::from_value(json).expect("команда разбирается обратно");
        assert_eq!(again, command);
    }
}

#[test]
fn command_json_from_the_interface_parses_by_name() {
    let command: Command =
        serde_json::from_str(r#"{"command":"set_speed","speed":3}"#).expect("команда интерфейса");
    assert_eq!(command, Command::SetSpeed { speed: 3 });
}

#[test]
fn a_new_life_opens_on_the_market_of_the_start_city() {
    let view = new_life().view();
    assert_eq!(view.screen, Screen::Market);
    assert_eq!(view.header.city, "Москва");
    assert_eq!(view.header.date, "1 января 1984, вс");
    assert_eq!(view.header.money, vec!["Наличные: 150 р."]);
    assert_eq!(view.market.place, "Толкучка у Рижского вокзала");
    assert_eq!(
        view.market.goods.len(),
        31,
        "из сорока товаров девять появятся позже 1984 года"
    );
    assert_eq!(view.pager, None, "пейджера в 1984 году нет");
    assert!(view.ironman, "жизнь начата в режиме ironman");
}

#[test]
fn a_new_life_requires_a_biography() {
    let settings = LifeSettings {
        biography: "  ".to_owned(),
        ..life_settings()
    };
    assert_eq!(
        Game::new_life(&settings, content()).err(),
        Some(CommandError::EmptyBiography)
    );
}

#[test]
fn speed_pause_and_screen_commands_change_the_snapshot() {
    let mut game = new_life();
    game.apply(Command::SetSpeed { speed: 3 })
        .expect("скорость 3 есть");
    game.apply(Command::Pause { paused: true })
        .expect("пауза ставится");
    game.apply(Command::OpenScreen {
        screen: Screen::Pager,
    })
    .expect("экран открывается");
    assert_eq!(game.view().time.speed, 3);
    assert!(game.view().time.paused);
    assert_eq!(game.view().screen, Screen::Pager);
}

#[test]
fn speed_outside_one_to_three_is_refused() {
    let mut game = new_life();
    assert_eq!(
        game.apply(Command::SetSpeed { speed: 4 }),
        Err(CommandError::InvalidSpeed { speed: 4, max: 3 })
    );
    assert_eq!(game.view().time.speed, 1);
}

#[test]
fn command_errors_serialize_with_their_kind() {
    let json = serde_json::to_value(CommandError::InvalidSpeed { speed: 9, max: 3 })
        .expect("ошибка сериализуется");
    assert_eq!(json["error"], "invalid_speed");
}
