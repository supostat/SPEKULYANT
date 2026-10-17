//! Один сид — одна история: одинаковый сценарий команд даёт побайтно одинаковый снимок мира.
//! Тест растёт вместе с подсистемами: каждая новая симуляция обязана сохранять это свойство.

use speculant_core::{Command, Game, LifeSettings, TimeRules};

const TIME_TOML: &str = include_str!("../../../data/epochs/time.toml");
const FRAME_MILLISECONDS: u32 = 16;
const DAYS: i32 = 400;

fn new_game(seed: u64) -> Game {
    let rules = TimeRules::from_toml(TIME_TOML).expect("data/epochs/time.toml валиден");
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, &rules).expect("жизнь начинается")
}

/// Сценарий игрока: кадры по 16 мс, смена скорости, пауза посреди года — до 400-го дня.
fn play_scripted_life(seed: u64) -> String {
    let mut game = new_game(seed);
    let mut paused_frames = 0;
    while game.today().day_index() < DAYS {
        let day = game.today().day_index();
        if day == 100 {
            game.apply(Command::SetSpeed { speed: 2 })
                .expect("скорость 2 есть");
        }
        if day == 200 && paused_frames == 0 {
            game.apply(Command::Pause { paused: true })
                .expect("пауза ставится");
        }
        if game.view().time.paused {
            paused_frames += 1;
            if paused_frames == 50 {
                game.apply(Command::Pause { paused: false })
                    .expect("пауза снимается");
            }
        }
        if day == 300 {
            game.apply(Command::SetSpeed { speed: 3 })
                .expect("скорость 3 есть");
        }
        game.advance(FRAME_MILLISECONDS);
    }
    serde_json::to_string(game.view()).expect("снимок сериализуется")
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
        serde_json::to_string(fine.view()).ok(),
        serde_json::to_string(coarse.view()).ok()
    );
}
