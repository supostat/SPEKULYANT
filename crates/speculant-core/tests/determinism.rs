//! Один сид — одна история: одинаковый сценарий команд даёт побайтно одинаковый снимок мира.
//! Тест растёт вместе с подсистемами: каждая новая симуляция обязана сохранять это свойство.

use std::path::Path;
use std::sync::Arc;

use speculant_core::time::GameDate;
use speculant_core::{Command, Content, Game, LifeSettings};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
const EVENTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/events");
const RIVALS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/npc");
const BUSINESSES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/business");
/// До конца августа 1985 года — через окно ящика «Столичной».
const EVENTFUL_DAYS: i32 = 600;
const FRAME_MILLISECONDS: u32 = 16;
const DAYS: i32 = 400;
/// Год своего дела — до 1 января 1985 года.
const BUSINESS_DAYS: i32 = 366;

fn new_game(seed: u64) -> Game {
    let content = Content::load(Path::new(DATA)).expect("data/ валиден");
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается")
}

/// Событие `data/` закрывается первым доступным выбором, который не берёт денег, не даёт срока и
/// не кончает жизнь, — такой есть у каждого события, и сценарий остаётся при своих деньгах и воле.
fn settle_harmlessly(game: &mut Game) {
    let pending = game.events().pending().expect("событие ждёт ответа");
    let choices = &game.content().events[&pending.event].choices;
    let shown = game.view().pending_event.expect("событие показано");
    let choice = shown
        .choices
        .iter()
        .zip(choices)
        .position(|(shown, choice)| {
            let consequences = &choice.consequences;
            shown.available
                && consequences.money.is_none_or(|money| !money.is_negative())
                && consequences.sentence_months.is_none()
                && consequences.end_life.is_none()
        })
        .expect("у события есть безобидный выбор")
        + 1;
    game.apply(Command::Choose { choice })
        .expect("доступный выбор принимается");
    game.apply(Command::Continue).expect("исход прочитан");
}

/// Действия игрока в сценарии: в какой день что сделать.
fn scripted_command(day: i32) -> Option<Command> {
    match day {
        10 => Some(Command::Buy {
            good: "chewing_gum".to_owned(),
            quantity: 3,
        }),
        60 => Some(Command::Deposit {
            place: "savings_bank".to_owned(),
            amount: 20,
        }),
        100 => Some(Command::SetSpeed { speed: 2 }),
        120 => Some(Command::Sell {
            good: "chewing_gum".to_owned(),
            quantity: 3,
        }),
        250 => Some(Command::Withdraw {
            place: "savings_bank".to_owned(),
            amount: 5,
        }),
        300 => Some(Command::SetSpeed { speed: 3 }),
        _ => None,
    }
}

/// Сценарий игрока: кадры по 16 мс, сделки, вклад, смена скорости, пауза посреди года — до
/// 400-го дня; события закрываются безобидным выбором.
fn play_scripted_life(seed: u64) -> String {
    let mut game = new_game(seed);
    let mut paused_frames = 0;
    let mut paused = false;
    let mut acted_on = -1;
    while game.today().day_index() < DAYS {
        if game.events().pending().is_some() {
            settle_harmlessly(&mut game);
            continue;
        }
        let day = game.today().day_index();
        if day != acted_on {
            acted_on = day;
            if let Some(command) = scripted_command(day) {
                let sale = matches!(command, Command::Sell { .. });
                game.apply(command).expect("команда сценария выполнима");
                assert!(
                    !sale
                        || game.view().messages.last().is_some_and(|message| {
                            message.starts_with("Продал: Жвачка, блок, 3 бл за")
                        }),
                    "продажа сценария дошла до мира"
                );
            }
        }
        if day == 200 && paused_frames == 0 {
            paused = true;
            game.apply(Command::Pause { paused })
                .expect("пауза ставится");
        }
        if paused {
            paused_frames += 1;
            if paused_frames == 50 {
                paused = false;
                game.apply(Command::Pause { paused })
                    .expect("пауза снимается");
            }
        }
        game.advance(FRAME_MILLISECONDS);
    }
    serde_json::to_string(&game.view()).expect("снимок сериализуется")
}

#[test]
fn the_same_seed_and_script_give_the_same_world_after_400_days() {
    let first = play_scripted_life(7);
    let second = play_scripted_life(7);
    assert_eq!(first, second);
    assert!(
        first.contains("\"date\":\"4 февраля 1985, пн\""),
        "400-й день после 1 января 1984"
    );
}

#[test]
fn another_seed_gives_another_market() {
    let first = play_scripted_life(7);
    let other = play_scripted_life(8);
    assert_ne!(first, other, "толкучка колеблется по сиду");
}

#[test]
fn frame_size_does_not_change_history() {
    let mut fine = new_game(7);
    let mut coarse = new_game(7);
    for (game, frame) in [(&mut fine, FRAME_MILLISECONDS), (&mut coarse, 1000)] {
        while game.today().day_index() < DAYS {
            if game.events().pending().is_some() {
                settle_harmlessly(game);
            } else {
                game.advance(frame);
            }
        }
    }
    assert_eq!(fine.today(), coarse.today());
    assert_eq!(fine.today().day_index(), DAYS);
    assert_eq!(
        serde_json::to_string(&fine.view()).ok(),
        serde_json::to_string(&coarse.view()).ok()
    );
}

/// Жизнь, полная событий: шанс события 0,3 в день, на каждое — первый доступный выбор. Итог —
/// снимок мира и журнал «дата: событие, выбор».
fn play_eventful_life(seed: u64) -> (String, Vec<String>) {
    let mut content = Content::load(Path::new(EVENTS)).expect("фикстура событий валидна");
    content.event_rules.daily_chance = 0.3;
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    let mut game = Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается");
    while game.today().day_index() < EVENTFUL_DAYS {
        game.advance(FRAME_MILLISECONDS);
        if game.events().pending().is_some() {
            let shown = game.view().pending_event.expect("событие ждёт ответа");
            let first = shown
                .choices
                .iter()
                .position(|choice| choice.available)
                .expect("хотя бы один выбор доступен");
            game.apply(Command::Choose { choice: first + 1 })
                .expect("доступный выбор принимается");
            game.apply(Command::Continue).expect("исход прочитан");
        }
    }
    let journal = game
        .journal()
        .iter()
        .map(|entry| {
            format!(
                "{}: {} {:?}",
                entry.date.to_iso(),
                entry.event,
                entry.choice
            )
        })
        .collect();
    (
        serde_json::to_string(&game.view()).expect("снимок сериализуется"),
        journal,
    )
}

#[test]
fn the_same_seed_gives_the_same_events_and_choices() {
    let (first_view, first_journal) = play_eventful_life(7);
    let (second_view, second_journal) = play_eventful_life(7);
    assert_eq!(first_journal, second_journal);
    assert_eq!(first_view, second_view);
    assert!(
        first_journal.len() > 100,
        "событий за 600 дней: {}",
        first_journal.len()
    );
    assert!(
        first_journal
            .iter()
            .any(|entry| entry.contains("vodka_crate")),
        "ящик «Столичной» в окне лета 1985-го"
    );
    let (_, other_journal) = play_eventful_life(8);
    assert_ne!(first_journal, other_journal, "другой сид — другие события");
}

/// Жизнь среди соперников: 300 дней торговли, поездок челноков, доходов и судеб. Итог — снимок
/// мира с запасами рынков и экраном соперников и капиталы соперников.
fn live_among_rivals(seed: u64) -> (String, Vec<String>) {
    let content = Content::load(Path::new(RIVALS)).expect("фикстура соперников валидна");
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    let mut game = Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается");
    for _ in 0..(1800 * 300 / FRAME_MILLISECONDS) {
        game.advance(FRAME_MILLISECONDS);
    }
    let capitals = game
        .rivals()
        .iter()
        .map(|rival| {
            format!(
                "{} {} {}",
                rival.npc(),
                rival.city(),
                rival.trader().money().cash().minor_units()
            )
        })
        .collect();
    (
        serde_json::to_string(&game.view()).expect("снимок сериализуется"),
        capitals,
    )
}

#[test]
fn the_same_seed_gives_the_same_rivals_and_their_deals() {
    let first = live_among_rivals(7);
    assert_eq!(first, live_among_rivals(7));
    assert_ne!(
        first.1,
        live_among_rivals(8).1,
        "другой сид — другие соперники"
    );
}

/// Год своего дела: палатка с управляющим, выручка с колебанием, наезды, проверки и уход
/// управляющего с шансами правил; на каждое событие — первый выбор. Итог — снимок мира с делом и
/// его отчётом.
fn run_a_business(seed: u64) -> String {
    let mut content = Content::load(Path::new(BUSINESSES)).expect("фикстура дел валидна");
    content.business.revenue_noise = 0.1;
    for (level, chance) in content
        .business
        .risk_levels
        .iter_mut()
        .zip([0.05, 0.1, 0.2, 0.3, 0.5])
    {
        level.monthly_chance = chance;
    }
    content.business.departure_chances = vec![0.1, 0.05, 0.02, 0.01, 0.0];
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    let mut game = Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается");
    game.apply(Command::OpenBusiness {
        kind: "stall".to_owned(),
    })
    .expect("палатка открывается");
    game.apply(Command::Hire {
        business: "stall@moscow".to_owned(),
        candidate: "gena_kabanov".to_owned(),
    })
    .expect("Гена свободен");
    while game.today().day_index() < BUSINESS_DAYS {
        game.advance(FRAME_MILLISECONDS);
        if game.events().pending().is_some() {
            game.apply(Command::Choose { choice: 1 })
                .expect("первый выбор доступен");
            game.apply(Command::Continue).expect("исход прочитан");
        }
    }
    serde_json::to_string(&game.view()).expect("снимок сериализуется")
}

#[test]
fn the_same_seed_gives_the_same_business_history() {
    let first = run_a_business(7);
    assert_eq!(first, run_a_business(7));
    assert!(
        first.contains("\"month\":\"декабрь 1984\""),
        "дело дожило до отчёта за декабрь"
    );
    assert_ne!(
        first,
        run_a_business(8),
        "другой сид — другая выручка и другие беды"
    );
}

/// История `data/` по сиду: таймлайн и мир до 1 июля 1985 года — через окно сухого закона;
/// события закрываются безобидным выбором.
fn live_through_the_dry_law(seed: u64) -> (Game, String) {
    let mut game = new_game(seed);
    while game.today() < GameDate::from_ymd(1985, 7, 1).expect("дата существует") {
        if game.events().pending().is_some() {
            settle_harmlessly(&mut game);
        } else {
            game.advance(1800);
        }
    }
    let view = serde_json::to_string(&game.view()).expect("снимок сериализуется");
    (game, view)
}

#[test]
fn the_same_seed_gives_the_same_history_and_its_news() {
    let (first, first_view) = live_through_the_dry_law(7);
    let (second, second_view) = live_through_the_dry_law(7);
    assert_eq!(first.timeline(), second.timeline());
    assert_eq!(first_view, second_view);
    assert!(
        first_view.contains("Указ о борьбе с пьянством"),
        "узел сухого закона случился в своё окно"
    );
    let (other, _) = live_through_the_dry_law(8);
    assert_ne!(
        first.timeline(),
        other.timeline(),
        "другой сид — другие даты, масштабы и исходы"
    );
}
