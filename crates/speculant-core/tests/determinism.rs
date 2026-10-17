//! Один сид — одна история: одинаковый сценарий команд даёт побайтно одинаковый снимок мира.
//! Тест растёт вместе с подсистемами: каждая новая симуляция обязана сохранять это свойство.

use std::path::Path;
use std::sync::Arc;

use speculant_core::{Command, Content, Game, LifeSettings};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
const FRAME_MILLISECONDS: u32 = 16;
const DAYS: i32 = 400;

fn new_game(seed: u64) -> Game {
    let content = Content::load(Path::new(DATA)).expect("data/ валиден");
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается")
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
/// 400-го дня.
fn play_scripted_life(seed: u64) -> String {
    let mut game = new_game(seed);
    let mut paused_frames = 0;
    let mut paused = false;
    let mut acted_on = -1;
    while game.today().day_index() < DAYS {
        let day = game.today().day_index();
        if day != acted_on {
            acted_on = day;
            if let Some(command) = scripted_command(day) {
                game.apply(command).expect("команда сценария выполнима");
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
    assert!(
        first.contains("Продал: Жвачка, блок, 3 бл за"),
        "сделки сценария дошли до мира"
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
    for _ in 0..(1800 * 400 / FRAME_MILLISECONDS) {
        fine.advance(FRAME_MILLISECONDS);
    }
    for _ in 0..720 {
        coarse.advance(1000);
    }
    assert_eq!(fine.today(), coarse.today());
    assert_eq!(fine.today().day_index(), DAYS);
    assert_eq!(
        serde_json::to_string(&fine.view()).ok(),
        serde_json::to_string(&coarse.view()).ok()
    );
}
