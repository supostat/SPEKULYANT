//! Метапрогрессия (§11.3, решение 15.10): из жизни в жизнь переходят только знания и открытия —
//! записная книжка товаров, людей и схем, открытые биографии, стартовые слухи об увиденной истории
//! и рекорды, раздельно ironman и свободный режим. С каталогом игрока всё это переживает и сеанс.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use speculant_core::content::{Content, EventId};
use speculant_core::save::Storage;
use speculant_core::time::GameDate;
use speculant_core::view::{NotebookEntryView, RecordView, StateView};
use speculant_core::{Command, CommandError, Game, LifeSettings};

const META: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/meta");
const DAY_MILLISECONDS: u32 = 1800;

fn load() -> Arc<Content> {
    Arc::new(Content::load(Path::new(META)).expect("фикстура метапрогрессии валидна"))
}

fn settings(biography: &str, seed: u64, ironman: bool) -> LifeSettings {
    LifeSettings {
        biography: biography.to_owned(),
        ironman,
        seed,
    }
}

fn student(seed: u64) -> LifeSettings {
    settings("student", seed, false)
}

/// Чистый каталог игрока для теста.
fn storage(test: &str) -> Storage {
    let directory = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
        .join("meta")
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

/// Событие с одним выбором — от начала до прочитанного исхода.
fn live_through(game: &mut Game, event: &str) {
    game.start_event(&EventId::new(event))
        .unwrap_or_else(|error| panic!("{event}: {error}"));
    play(game, Command::Choose { choice: 1 });
    play(game, Command::Continue);
}

fn live_until(game: &mut Game, until: GameDate) {
    while game.today() < until {
        assert!(game.advance(DAY_MILLISECONDS), "время идёт");
    }
}

fn retire(game: &mut Game) {
    play(
        game,
        Command::Retire {
            ending: "rest".to_owned(),
        },
    );
}

/// Ценник толкучки за штуку: (покупка, продажа).
fn tags(view: &StateView, good: &str) -> (String, String) {
    let row = view
        .market
        .goods
        .iter()
        .find(|row| row.good == good)
        .expect("товар на толкучке");
    (row.buy.clone(), row.sell.clone())
}

fn trade(game: &mut Game, buy: bool, good: &str) {
    let good = good.to_owned();
    let command = if buy {
        Command::Buy { good, quantity: 1 }
    } else {
        Command::Sell { good, quantity: 1 }
    };
    play(game, command);
}

fn entry(topic: &str, title: &str, text: &str) -> NotebookEntryView {
    NotebookEntryView {
        topic: topic.to_owned(),
        title: title.to_owned(),
        text: text.to_owned(),
    }
}

#[test]
fn the_notebook_keeps_goods_people_and_schemes_across_lives() {
    let mut game = Game::new_life(&student(7), load()).expect("жизнь начинается");
    assert!(
        game.view().notebook.is_empty(),
        "первая жизнь ничего не знает"
    );
    let (jeans_buy, _) = tags(&game.view(), "jeans");
    trade(&mut game, true, "jeans");
    live_through(&mut game, "kofi_favor");
    live_through(&mut game, "windfall");
    let (_, jeans_sell) = tags(&game.view(), "jeans");
    trade(&mut game, false, "jeans");
    let (_, gum_sell) = tags(&game.view(), "chewing_gum");
    trade(&mut game, false, "chewing_gum");
    assert!(
        game.view().notebook.is_empty(),
        "книжка — знания прошлых жизней"
    );
    retire(&mut game);

    let gum_profit: u32 = gum_sell.parse().expect("ценник жвачки — целые рубли");
    let expected = vec![
        entry(
            "Товары",
            "Жвачка, блок",
            &format!("дороже всего продал по {gum_sell} р. (Москва, 1984)"),
        ),
        entry(
            "Товары",
            "Джинсы (фирма)",
            &format!(
                "дешевле всего купил по {jeans_buy} р. (Москва, 1984); дороже всего продал по {jeans_sell} р. (Москва, 1984)"
            ),
        ),
        entry(
            "Люди",
            "Кофи",
            "сокурсник из Ганы. 01.01.1984: Кофи выручил",
        ),
        entry(
            "Схемы",
            "Жвачка, блок",
            &format!("лучшая сделка: +{gum_profit} р. в 1984 году"),
        ),
    ];
    assert_eq!(game.view().notebook, expected);

    play(&mut game, Command::NewLife(student(7)));
    assert_eq!(
        game.view().notebook,
        expected,
        "книжка переходит в новую жизнь"
    );
    trade(&mut game, true, "jeans");
    trade(&mut game, true, "jeans");
    retire(&mut game);
    assert_eq!(
        game.view().notebook,
        expected,
        "дороже купленная пара не вытесняет самую дешёвую из прошлой жизни"
    );

    play(&mut game, Command::NewLife(student(7)));
    trade(&mut game, true, "jeans");
    trade(&mut game, false, "jeans");
    retire(&mut game);
    let schemes: Vec<NotebookEntryView> = game
        .view()
        .notebook
        .into_iter()
        .filter(|entry| entry.topic == "Схемы")
        .collect();
    assert_eq!(schemes, [expected[3].clone()], "сделка в убыток — не схема");
}

#[test]
fn unlocks_and_knowledge_outlive_the_session_on_disk() {
    let content = load();
    let storage = storage("unlocks");
    let mut first =
        Game::open(&student(7), Arc::clone(&content), storage.clone()).expect("жизнь начинается");
    live_through(&mut first, "nomenklatura_offer");
    live_through(&mut first, "kofi_favor");
    retire(&mut first);

    assert!(matches!(
        Game::new_life(&settings("party_son", 7, false), Arc::clone(&content)),
        Err(CommandError::BiographyLocked { .. })
    ));
    let next = Game::open(&settings("party_son", 8, false), content, storage)
        .expect("открытая биография — с диска");
    assert_eq!(next.character().biography().as_str(), "party_son");
    assert_eq!(next.view().notebook, first.view().notebook);
    assert_eq!(next.view().records, first.view().records);
}

#[test]
fn starting_rumors_tell_what_past_lives_saw() {
    let content = load();
    let mut game = Game::new_life(&student(7), Arc::clone(&content)).expect("жизнь начинается");
    retire(&mut game);
    play(&mut game, Command::NewLife(student(8)));
    assert!(
        game.view().rumors.is_empty(),
        "прошлая жизнь кончилась в первый день и ничего не видела"
    );

    live_until(&mut game, GameDate::from_ymd(1984, 3, 1).expect("дата"));
    retire(&mut game);
    play(&mut game, Command::NewLife(student(9)));
    assert_eq!(
        game.view().rumors,
        ["Говорят, в 1984 году — Джинсовая лихорадка."]
    );
    let node = game
        .timeline()
        .nodes()
        .iter()
        .find(|node| node.node.as_str() == "jeans_fever")
        .expect("узел в таймлайне")
        .date;
    live_until(&mut game, node);
    assert!(game.view().rumors.is_empty(), "случившееся — уже не слух");
}

#[test]
fn records_keep_the_richest_lives_of_each_mode_apart() {
    let mut content = Content::load(Path::new(META)).expect("фикстура метапрогрессии валидна");
    content.life.records_size = 2;
    let mut game =
        Game::new_life(&settings("student", 1, true), Arc::new(content)).expect("жизнь начинается");
    let mut lives = Vec::new();
    for seed in 1..=3 {
        play(&mut game, Command::NewLife(settings("student", seed, true)));
        match seed {
            1 => {}
            2 => live_through(&mut game, "windfall"),
            _ => trade(&mut game, true, "jeans"),
        }
        retire(&mut game);
        let line = &game.view().summary.expect("итоги").lines[0];
        lives.push((game.character().name().to_owned(), line.value.clone()));
    }
    play(&mut game, Command::NewLife(settings("student", 4, false)));
    retire(&mut game);

    let [plain, gifted, spent] = [&lives[0], &lives[1], &lives[2]];
    assert_ne!(
        plain.1, spent.1,
        "разница ценников джинсов съела часть капитала"
    );
    let row = |(name, capital): &(String, String)| RecordView {
        name: name.clone(),
        biography: "Студент".to_owned(),
        capital: capital.clone(),
        years: "1984–1984".to_owned(),
        ending: "Уйти на покой".to_owned(),
    };
    let records = game.view().records;
    assert_eq!(
        records.ironman,
        [row(gifted), row(plain)],
        "в «Рейтинг Форбс» — две богатейшие ironman-жизни: с подарком и без трат"
    );
    assert_eq!(records.free.len(), 1, "свободная жизнь — в своей таблице");
    assert_eq!(records.free[0].name, game.character().name());
}
