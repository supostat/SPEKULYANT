//! Миры для тестов ботов: настоящие данные и фикстуры ядра.

use std::path::Path;
use std::sync::Arc;

use speculant_core::time::GameDate;
use speculant_core::{Command, Content, Game, LifeSettings};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
const CORE_FIXTURES: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../speculant-core/tests/fixtures"
);
const SEED: u64 = 7;
const SPEED: u8 = 1;

pub(crate) fn real_data() -> Arc<Content> {
    Arc::new(Content::load(Path::new(DATA)).expect("data/ валиден"))
}

pub(crate) fn core_fixture(name: &str) -> Arc<Content> {
    let path = Path::new(CORE_FIXTURES).join(name);
    Arc::new(Content::load(&path).unwrap_or_else(|errors| panic!("фикстура {name}: {errors:?}")))
}

pub(crate) fn new_life(content: &Arc<Content>, biography: &str) -> Game {
    let settings = LifeSettings {
        biography: biography.to_owned(),
        ironman: true,
        seed: SEED,
    };
    Game::new_life(&settings, Arc::clone(content)).expect("биография открыта с начала")
}

pub(crate) fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

/// Живёт до даты по дню; событие, если придёт, закрывает безобидным выбором.
pub(crate) fn live_until(game: &mut Game, until: GameDate) {
    let day = game.content().time.clock.day_milliseconds_at(SPEED);
    while game.today() < until {
        assert!(
            game.summary().is_none(),
            "жизнь кончилась раньше {}",
            until.long()
        );
        if game.events().pending().is_some() {
            settle_harmlessly(game);
            continue;
        }
        game.advance(day);
    }
}

/// Закрывает ожидающее событие первым доступным выбором, который не забирает ни денег, ни
/// товара, не даёт срока и не кончает жизнь, — тест сам распоряжается кошельком и карманом.
pub(crate) fn settle_harmlessly(game: &mut Game) {
    let pending = game.events().pending().expect("событие ждёт");
    let choices = &game.content().events[&pending.event].choices;
    let event = game.view().pending_event.expect("событие показано");
    if event.outcome.is_none() {
        let choice = event
            .choices
            .iter()
            .zip(choices)
            .position(|(shown, choice)| {
                let consequences = &choice.consequences;
                shown.available
                    && consequences.money.is_none_or(|money| !money.is_negative())
                    && consequences.goods.iter().all(|change| change.change >= 0)
                    && consequences.sentence_months.is_none()
                    && consequences.end_life.is_none()
            })
            .expect("у события есть безобидный выбор")
            + 1;
        game.apply(Command::Choose { choice })
            .expect("выбор доступен");
    }
    game.apply(Command::Continue).expect("исход прочитан");
}
