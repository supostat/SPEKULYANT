//! Пейджер (§13.2): он виден, пока устройство лежит в кармане, и в ленту попадают сообщения,
//! пришедшие при нём, — здесь новости рынка от узлов истории.

use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{Content, GoodId};
use speculant_core::life::PagerDevice;
use speculant_core::time::GameDate;
use speculant_core::view::{PagerMessageView, PagerView};
use speculant_core::{Command, CommandError, Game, LifeSettings};

const HISTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/history");
const DAY_MILLISECONDS: u32 = 1800;
const SEED: u64 = 7;

/// Фикстура истории без случайных событий, где пейджером служит жвачка.
fn game() -> Game {
    let mut content = Content::load(Path::new(HISTORY)).expect("фикстура истории валидна");
    content.event_rules.daily_chance = 0.0;
    content.life.pager.devices = vec![PagerDevice {
        good: GoodId::new("chewing_gum"),
        title: "ПЕЙДЖЕР · абонент {number}".to_owned(),
    }];
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: false,
        seed: SEED,
    };
    Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается")
}

fn buy_the_device(game: &mut Game) {
    game.apply(Command::Buy {
        good: "chewing_gum".to_owned(),
        quantity: 1,
    })
    .expect("жвачка в продаже");
}

/// Живёт до даты; событие, если придёт, закрывает последним выбором.
fn live_until(game: &mut Game, until: GameDate) {
    while game.today() < until {
        if let Some(event) = game.view().pending_event {
            game.apply(Command::Choose {
                choice: event.choices.len(),
            })
            .expect("последний выбор доступен");
            game.apply(Command::Continue).expect("исход прочитан");
            continue;
        }
        game.advance(DAY_MILLISECONDS);
    }
}

fn after_the_denomination() -> GameDate {
    GameDate::from_ymd(1990, 1, 2).expect("дата существует")
}

#[test]
fn the_pager_shows_only_with_a_device_in_the_pocket() {
    let mut game = game();
    assert_eq!(game.view().pager, None);
    buy_the_device(&mut game);
    assert_eq!(
        game.view().pager,
        Some(PagerView {
            device: "ПЕЙДЖЕР · абонент 1007".to_owned(),
            messages: Vec::new(),
        })
    );
}

#[test]
fn market_news_reach_the_pager_while_the_device_is_held() {
    let mut game = game();
    buy_the_device(&mut game);
    live_until(&mut game, after_the_denomination());
    let pager = game.view().pager.expect("пейджер в кармане");
    assert_eq!(
        pager.messages.last(),
        Some(&PagerMessageView {
            time: "01.01".to_owned(),
            text: "ДЕНОМИНАЦИЯ: ТРИ НУЛЯ ДОЛОЙ.".to_owned(),
            urgent: false,
        })
    );
}

#[test]
fn news_without_a_device_pass_the_pager_by() {
    let mut game = game();
    live_until(&mut game, after_the_denomination());
    buy_the_device(&mut game);
    assert_eq!(
        game.view().pager.map(|pager| pager.messages),
        Some(Vec::new())
    );
}

#[test]
fn an_erased_message_leaves_the_feed_for_good() {
    let mut game = game();
    buy_the_device(&mut game);
    live_until(&mut game, after_the_denomination());
    let before = game.view().pager.expect("пейджер в кармане").messages;
    assert!(before.len() > 1, "в ленте несколько новостей");
    game.apply(Command::ErasePagerMessage { message: 0 })
        .expect("самое старое сообщение стирается");
    assert_eq!(
        game.view().pager.expect("пейджер в кармане").messages,
        before[1..]
    );
    let left = before.len() - 1;
    assert_eq!(
        game.apply(Command::ErasePagerMessage { message: left }),
        Err(CommandError::NoSuchPagerMessage {
            message: left,
            count: left,
        })
    );
}
