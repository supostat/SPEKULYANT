//! Сохранения (§11.4, решение 15.10): загруженная жизнь показывает побайтно тот же снимок и живёт
//! дальше так же, как несохранённая; ironman сохраняется сам после каждого дня и поступка, не
//! откатывается и стирается в финале; свободный режим пишет именованные слоты; файлы версии 1
//! читаются, файлы версии 4 берут дрейф цен из сида мира, а файлы новее, испорченные и сделанные с
//! другими данными — нет.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use speculant_core::content::{Content, EventId, GoodId};
use speculant_core::save::{LIFE, Storage};
use speculant_core::time::GameDate;
use speculant_core::{Command, CommandError, Game, LifeSettings};

const BUSINESS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/business");
const SAVE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/save");
/// Каталог игрока, снятый форматом версии 1 с фикстуры `save`: жизнь в слоте `golden` и
/// метапрогрессия с одним рекордом. Меняется формат — растёт версия и пишется шаг миграции, а
/// этот каталог остаётся как есть; меняется сама фикстура `save` — каталог снимается заново
/// той же версией формата.
const VERSION_1: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/save-v1");
const FRAME_MILLISECONDS: u32 = 16;
const DAY_MILLISECONDS: u32 = 1800;

/// Фикстура дел с колебаниями выручки, наездами и уходами управляющих — мир, которому есть что
/// терять при сохранении.
fn lively_business() -> Arc<Content> {
    let mut content = Content::load(Path::new(BUSINESS)).expect("фикстура дел валидна");
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
    Arc::new(content)
}

fn settings(seed: u64, ironman: bool) -> LifeSettings {
    LifeSettings {
        biography: "student".to_owned(),
        ironman,
        seed,
    }
}

/// Чистый каталог игрока для теста.
fn storage(test: &str) -> Storage {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("save")
        .join(test);
    if directory.exists() {
        std::fs::remove_dir_all(&directory).expect("прошлый каталог теста стирается");
    }
    Storage::new(directory)
}

fn play(game: &mut Game, command: Command) {
    let shown = format!("{command:?}");
    game.apply(command)
        .unwrap_or_else(|error| panic!("{shown}: {error}"));
}

fn view_json(game: &Game) -> String {
    serde_json::to_string(&game.view()).expect("снимок сериализуется")
}

fn open_stall(game: &mut Game) {
    play(
        game,
        Command::OpenBusiness {
            kind: "stall".to_owned(),
        },
    );
    play(
        game,
        Command::Hire {
            business: "stall@moscow".to_owned(),
            candidate: "gena_kabanov".to_owned(),
        },
    );
    play(
        game,
        Command::Buy {
            good: "jeans".to_owned(),
            quantity: 2,
        },
    );
}

/// Кадрами по 16 мс до дня `day` от 1 января 1984 года; на событие — первый выбор.
fn live_frames_until(game: &mut Game, day: i32) {
    while game.today().day_index() < day {
        if game.events().pending().is_some() {
            play(game, Command::Choose { choice: 1 });
            play(game, Command::Continue);
        }
        game.advance(FRAME_MILLISECONDS);
    }
}

#[test]
fn a_loaded_life_shows_the_same_view_and_lives_on_the_same_way() {
    let content = lively_business();
    let storage = storage("same_future");
    let mut game = Game::open(&settings(7, false), Arc::clone(&content), storage.clone())
        .expect("жизнь начинается");
    open_stall(&mut game);
    live_frames_until(&mut game, 75);
    play(&mut game, Command::SetSpeed { speed: 2 });
    game.advance(FRAME_MILLISECONDS * 7);
    play(
        &mut game,
        Command::Save {
            slot: "march".to_owned(),
        },
    );
    let saved = view_json(&game);

    let mut resumed =
        Game::resume(Arc::clone(&content), storage.clone(), "march").expect("слот читается");
    assert_eq!(view_json(&resumed), saved, "снимок побайтно тот же");

    let mut loaded = Game::open(&settings(99, false), Arc::clone(&content), storage)
        .expect("другая жизнь начинается");
    play(
        &mut loaded,
        Command::Load {
            slot: "march".to_owned(),
        },
    );
    assert_eq!(view_json(&loaded), saved, "загрузка заменяет жизнь целиком");

    for world in [&mut game, &mut resumed, &mut loaded] {
        live_frames_until(world, 200);
    }
    let future = view_json(&game);
    assert_ne!(future, saved, "мир ушёл вперёд");
    assert_eq!(
        view_json(&resumed),
        future,
        "случай мира, часы с остатком кадра и дела восстановлены целиком"
    );
    assert_eq!(view_json(&loaded), future);
}

#[test]
fn ironman_saves_itself_cannot_roll_back_and_is_erased_at_the_end() {
    let content = Arc::new(Content::load(Path::new(SAVE)).expect("фикстура сохранений валидна"));
    let storage = storage("ironman");
    let resumed =
        |storage: &Storage| Game::resume(Arc::clone(&content), storage.clone(), "ironman");
    let mut game = Game::open(&settings(7, true), Arc::clone(&content), storage.clone())
        .expect("жизнь начинается");
    assert!(game.is_ironman());
    assert_eq!(
        view_json(&resumed(&storage).expect("слот ironman пишется при рождении")),
        view_json(&game)
    );

    assert!(game.advance(DAY_MILLISECONDS));
    assert_eq!(
        view_json(&resumed(&storage).expect("слот")),
        view_json(&game),
        "день прошёл — жизнь сохранилась"
    );
    open_stall(&mut game);
    assert_eq!(
        view_json(&resumed(&storage).expect("слот")),
        view_json(&game),
        "поступок — и жизнь сохранилась"
    );
    assert_eq!(
        game.apply(Command::Save {
            slot: "spare".to_owned()
        }),
        Err(CommandError::IronmanSavesItself)
    );
    assert_eq!(
        game.apply(Command::Load {
            slot: "ironman".to_owned()
        }),
        Err(CommandError::IronmanNoRollback)
    );

    game.start_event(&EventId::new("heart_attack"))
        .expect("событие финала");
    play(&mut game, Command::Choose { choice: 1 });
    play(&mut game, Command::Continue);
    assert!(game.summary().is_some());
    assert_eq!(
        resumed(&storage).err(),
        Some(CommandError::NoSuchSave {
            slot: "ironman".to_owned()
        }),
        "кончившаяся ironman-жизнь стёрта"
    );
    let next = Game::open(&settings(8, true), content, storage).expect("новая жизнь");
    assert_eq!(
        next.view().records.ironman.len(),
        1,
        "жизнь легла в «Рейтинг Форбс»"
    );
}

#[test]
fn free_saves_keep_to_their_rules() {
    let content = Arc::new(Content::load(Path::new(SAVE)).expect("фикстура сохранений валидна"));
    let mut in_memory =
        Game::new_life(&settings(7, false), Arc::clone(&content)).expect("жизнь начинается");
    for command in [
        Command::Save {
            slot: "first".to_owned(),
        },
        Command::Load {
            slot: "first".to_owned(),
        },
    ] {
        assert_eq!(in_memory.apply(command), Err(CommandError::NoStorage));
    }

    let mut game =
        Game::open(&settings(7, false), content, storage("free_rules")).expect("жизнь начинается");
    for slot in ["", "Первый", "../evil", "ironman", "a".repeat(33).as_str()] {
        assert_eq!(
            game.apply(Command::Save {
                slot: slot.to_owned()
            }),
            Err(CommandError::InvalidSlot {
                slot: slot.to_owned()
            }),
            "слот «{slot}»"
        );
    }
    assert_eq!(
        game.apply(Command::Load {
            slot: "nothing".to_owned()
        }),
        Err(CommandError::NoSuchSave {
            slot: "nothing".to_owned()
        })
    );
    play(
        &mut game,
        Command::Save {
            slot: "first".to_owned(),
        },
    );
    game.start_event(&EventId::new("heart_attack"))
        .expect("событие финала");
    play(&mut game, Command::Choose { choice: 1 });
    play(&mut game, Command::Continue);
    assert_eq!(
        game.apply(Command::Save {
            slot: "after".to_owned()
        }),
        Err(CommandError::LifeOver)
    );
    play(
        &mut game,
        Command::Load {
            slot: "first".to_owned(),
        },
    );
    assert_eq!(
        game.summary(),
        None,
        "из итогов можно вернуться в сохранённую жизнь"
    );
}

#[test]
fn quitting_puts_an_ironman_life_away_and_leaves_a_free_one_unsaved() {
    let content = Arc::new(Content::load(Path::new(SAVE)).expect("фикстура сохранений валидна"));
    let ironman_place = storage("quit_ironman");
    let mut ironman = Game::open(
        &settings(7, true),
        Arc::clone(&content),
        ironman_place.clone(),
    )
    .expect("ironman-жизнь начинается");
    play(&mut ironman, Command::SetSpeed { speed: 3 });
    play(&mut ironman, Command::Quit);
    let resumed = Game::resume(Arc::clone(&content), ironman_place, "ironman")
        .expect("ironman-жизнь в слоте");
    assert_eq!(resumed.view().time.speed, 3, "слот дописан при выходе");
    let free_place = storage("quit_free");
    let mut free = Game::open(
        &settings(7, false),
        Arc::clone(&content),
        free_place.clone(),
    )
    .expect("свободная жизнь начинается");
    play(&mut free, Command::Quit);
    assert_eq!(
        Game::resume(content, free_place, "ironman").err(),
        Some(CommandError::NoSuchSave {
            slot: "ironman".to_owned()
        }),
        "свободная жизнь при выходе сама не сохраняется"
    );
}

#[test]
fn a_version_one_directory_still_loads() {
    let content = Arc::new(Content::load(Path::new(SAVE)).expect("фикстура сохранений валидна"));
    let mut game =
        Game::resume(content, Storage::new(VERSION_1), "golden").expect("файлы версии 1 читаются");
    assert_eq!(game.today(), GameDate::from_ymd(1984, 2, 10).expect("дата"));
    let view = game.view();
    assert_eq!(view.businesses.len(), 1);
    assert_eq!(
        view.businesses[0]
            .manager
            .as_ref()
            .map(|manager| manager.name.as_str()),
        Some("Гена Кабанов")
    );
    assert_eq!(view.records.free.len(), 1, "рекорд из файла метапрогрессии");
    assert_eq!(
        view.pager, None,
        "лента пейджера версии 1 пуста, а устройства в наборе нет"
    );
    assert!(
        game.advance(DAY_MILLISECONDS),
        "жизнь из старого файла идёт"
    );
}

#[test]
fn a_version_four_life_takes_its_price_drift_from_the_world_seed() {
    let mut content = Content::load(Path::new(SAVE)).expect("фикстура сохранений валидна");
    content.drift.amplitude = 0.3;
    let content = Arc::new(content);
    let storage = storage("version_four");
    let mut game = Game::open(&settings(7, false), Arc::clone(&content), storage.clone())
        .expect("жизнь начинается");
    live_frames_until(&mut game, 40);
    play(
        &mut game,
        Command::Save {
            slot: "five".to_owned(),
        },
    );
    let jeans = GoodId::new("jeans");
    let item = &content.goods[&jeans];
    let curve = item
        .curve
        .kopecks_on(game.today())
        .expect("джинсы продаются")
        * content.cities[game.city()].multiplier(&item.category);
    let normal = game
        .markets()
        .normal_kopecks(&content, game.city(), &jeans, game.today())
        .expect("джинсы продаются");
    assert!(
        (normal / curve - 1.0).abs() > 1e-6,
        "цена в этом мире ушла от кривой"
    );

    let mut file: serde_json::Value =
        serde_json::from_str(&storage.read_life("five").expect("файл версии 5"))
            .expect("файл жизни — JSON");
    file["version"] = 4.into();
    file["body"]["markets"]
        .as_object_mut()
        .expect("рынки в теле")
        .remove("drift")
        .expect("дрейф в файле версии 5");
    storage
        .write_life("four", &file.to_string())
        .expect("файл пишется");
    let mut resumed =
        Game::resume(Arc::clone(&content), storage, "four").expect("файл версии 4 читается");
    assert_eq!(
        view_json(&resumed),
        view_json(&game),
        "цены те же: дрейф взял сид мира"
    );
    live_frames_until(&mut game, 120);
    live_frames_until(&mut resumed, 120);
    assert_eq!(
        view_json(&resumed),
        view_json(&game),
        "и дальше идут так же"
    );
}

#[test]
fn newer_broken_and_foreign_files_are_refused() {
    let save_content =
        Arc::new(Content::load(Path::new(SAVE)).expect("фикстура сохранений валидна"));
    let golden = Storage::new(VERSION_1)
        .read_life("golden")
        .expect("файл версии 1");
    let storage = storage("refused");
    let newer = LIFE.version + 1;
    for (slot, text) in [
        (
            "newer",
            golden.replace("\"version\":1", &format!("\"version\":{newer}")),
        ),
        ("broken", golden[..golden.len() / 2].to_owned()),
        ("meta", golden.replace("speculant-life", "speculant-meta")),
    ] {
        storage.write_life(slot, &text).expect("файл пишется");
    }
    let reason = |slot: &str, content: &Arc<Content>| match Game::resume(
        Arc::clone(content),
        storage.clone(),
        slot,
    ) {
        Err(CommandError::SaveUnreadable { reason, .. }) => reason,
        other => panic!("{slot}: {other:?}"),
    };
    assert_eq!(
        reason("newer", &save_content),
        format!(
            "файл версии {newer}, а игра читает до версии {}",
            LIFE.version
        )
    );
    assert!(reason("broken", &save_content).starts_with("файл испорчен"));
    assert_eq!(reason("meta", &save_content), "это не тот файл");
    storage.write_life("golden", &golden).expect("файл пишется");
    assert_eq!(
        reason("golden", &lively_business()),
        "файл сделан с другими данными игры"
    );
}
