//! Конвертации моста без Godot: JSON интерфейса ↔ типы ядра. Отделены от класса Godot,
//! чтобы проверяться обычными тестами.
//!
//! Ответ на команду — `{"ok":true}` или `{"error":"<вид>", ..., "message":"<текст для игрока>"}`.
//!
//! Старт сеанса — `{"storage": "<каталог игрока>", "life": {"biography", "ironman", "seed"}}`,
//! `{"storage": "<каталог игрока>", "resume": "<слот>"}` или `{"storage": "<каталог игрока>"}` —
//! лобби: каталог открыт, жизнь не начата, снимок — биографии, записная книжка и рекорды. В
//! каталоге ядро само пишет жизни `saves/<слот>.spk` и метапрогрессию `meta.spk`; без `storage`
//! жизнь идёт в памяти и не сохраняется, а лобби без каталога не бывает.

use serde::Deserialize;
use serde_json::{Value, json};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use include_dir::{Dir, include_dir};
use speculant_core::content::{ContentError, ContentFile};
use speculant_core::save::Storage;
use speculant_core::{Command, CommandError, Content, Game, LifeSettings, Lobby};

pub const OK: &str = r#"{"ok":true}"#;
static DATA: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/../../data");
static CONTENT: OnceLock<Result<Arc<Content>, String>> = OnceLock::new();

/// Контент игры из вшитого при сборке `data/`: разбирается один раз на запуск и делится жизнями.
fn content() -> Result<Arc<Content>, String> {
    CONTENT
        .get_or_init(|| {
            let mut files = Vec::new();
            let mut errors = Vec::new();
            collect_files(&DATA, &mut files, &mut errors);
            if errors.is_empty() {
                Content::from_files(files)
                    .map(Arc::new)
                    .map_err(|content_errors| invalid_content(&content_errors))
            } else {
                Err(invalid_content(&errors))
            }
        })
        .clone()
}

fn invalid_content(errors: &[ContentError]) -> String {
    let lines: Vec<String> = errors.iter().map(ContentError::to_string).collect();
    bridge_error("invalid_content", lines.join("\n"))
}

fn collect_files(dir: &Dir<'_>, files: &mut Vec<ContentFile>, errors: &mut Vec<ContentError>) {
    for file in dir.files() {
        let path = slash_path(file.path());
        match file.contents_utf8() {
            Some(text) => files.push(ContentFile::new(path, text)),
            None => errors.push(ContentError {
                file: path,
                line: None,
                message: "файл не в UTF-8".to_owned(),
            }),
        }
    }
    for subdirectory in dir.dirs() {
        collect_files(subdirectory, files, errors);
    }
}

/// Путь относительно `data/` через `/` на любой платформе — так его ждёт загрузчик контента.
fn slash_path(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/")
}

/// Сеанс моста: лобби, пока жизнь не начата, или идущая жизнь.
#[derive(Debug)]
pub enum Session {
    Lobby(Lobby),
    Life(Box<Game>),
}

/// С чего начинается сеанс: каталог игрока, если он есть, и новая жизнь, слот или ничего — лобби.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Start {
    storage: Option<PathBuf>,
    life: Option<LifeSettings>,
    resume: Option<String>,
}

pub fn start(start_json: &str) -> Result<Session, String> {
    let start: Start = serde_json::from_str(start_json)
        .map_err(|error| bridge_error("malformed_settings", error.to_string()))?;
    let content = content()?;
    let storage = start.storage.map(Storage::new);
    let session = match (start.life, start.resume, storage) {
        (Some(_), Some(_), _) => {
            return Err(bridge_error(
                "malformed_settings",
                "жизнь начинается либо заново, либо из слота".to_owned(),
            ));
        }
        (Some(settings), None, None) => {
            Game::new_life(&settings, content).map(|game| Session::Life(Box::new(game)))
        }
        (Some(settings), None, Some(storage)) => {
            Game::open(&settings, content, storage).map(|game| Session::Life(Box::new(game)))
        }
        (None, Some(slot), Some(storage)) => {
            Game::resume(content, storage, &slot).map(|game| Session::Life(Box::new(game)))
        }
        (None, None, Some(storage)) => Lobby::open(content, &storage).map(Session::Lobby),
        (None, _, None) => Err(CommandError::NoStorage),
    };
    session.map_err(|error| command_error(&error))
}

pub fn apply(game: &mut Game, command_json: &str) -> String {
    let command: Command = match serde_json::from_str(command_json) {
        Ok(command) => command,
        Err(error) => return bridge_error("malformed_command", error.to_string()),
    };
    match game.apply(command) {
        Ok(()) => OK.to_owned(),
        Err(error) => command_error(&error),
    }
}

pub fn advance(game: &mut Game, delta_ms: i64) -> Result<bool, String> {
    let milliseconds = u32::try_from(delta_ms).map_err(|_| {
        bridge_error(
            "invalid_delta",
            format!("шаг времени {delta_ms} мс вне 0..{}", u32::MAX),
        )
    })?;
    Ok(game.advance(milliseconds))
}

pub fn view_json(game: &Game) -> String {
    serde_json::to_string(&game.view()).expect("StateView сериализуется в JSON")
}

pub fn lobby_view_json(lobby: &Lobby) -> String {
    serde_json::to_string(&lobby.view()).expect("LobbyView сериализуется в JSON")
}

pub fn no_life() -> String {
    bridge_error(
        "no_life",
        "жизнь ещё не начата: сначала new_life".to_owned(),
    )
}

fn bridge_error(kind: &str, message: String) -> String {
    json!({ "error": kind, "message": message }).to_string()
}

fn command_error(error: &CommandError) -> String {
    let mut body = serde_json::to_value(error).expect("CommandError сериализуется в JSON");
    if let Value::Object(fields) = &mut body {
        fields.insert("message".to_owned(), Value::String(error.to_string()));
    }
    body.to_string()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use serde_json::Value;
    use speculant_core::{Command, Game, LobbyView, StateView};

    use super::{OK, Session, advance, apply, lobby_view_json, no_life, start, view_json};

    const LIFE: &str = r#"{"life":{"biography":"student","ironman":true,"seed":7}}"#;

    /// JSON каждой команды интерфейса — так их шлёт GDScript.
    const COMMANDS: [&str; 31] = [
        r#"{"command":"new_life","biography":"engineer","ironman":false,"seed":3}"#,
        r#"{"command":"advance","milliseconds":1800}"#,
        r#"{"command":"set_speed","speed":2}"#,
        r#"{"command":"pause","paused":true}"#,
        r#"{"command":"fast_forward","until":"until_event"}"#,
        r#"{"command":"buy","good":"chewing_gum","quantity":1}"#,
        r#"{"command":"sell","good":"chewing_gum","quantity":1}"#,
        r#"{"command":"travel","destination":"leningrad"}"#,
        r#"{"command":"choose","choice":1}"#,
        r#"{"command":"continue"}"#,
        r#"{"command":"open_screen","screen":"pager"}"#,
        r#"{"command":"open_business","kind":"sewing_workshop"}"#,
        r#"{"command":"set_policy","business":"sewing_workshop@moscow","policy":"markup","option":1}"#,
        r#"{"command":"hire","business":"sewing_workshop@moscow","candidate":"gena_kabanov"}"#,
        r#"{"command":"fire","business":"sewing_workshop@moscow","employee":"gena_kabanov"}"#,
        r#"{"command":"call","contact":"kofi"}"#,
        r#"{"command":"erase_pager_message","message":0}"#,
        r#"{"command":"buy_housing","housing":"communal_room"}"#,
        r#"{"command":"retire","ending":"rest"}"#,
        r#"{"command":"set_work","work":true}"#,
        r#"{"command":"use_service","service":"infirmary"}"#,
        r#"{"command":"order_parcel"}"#,
        r#"{"command":"apply_for_parole"}"#,
        r#"{"command":"bribe_for_release"}"#,
        r#"{"command":"deposit","place":"savings_bank","amount":10}"#,
        r#"{"command":"withdraw","place":"savings_bank","amount":10}"#,
        r#"{"command":"save","slot":"first"}"#,
        r#"{"command":"load","slot":"first"}"#,
        r#"{"command":"quit"}"#,
        r#"{"command":"pause","paused":false}"#,
        r#"{"command":"fast_forward","until":"until_arrival"}"#,
    ];

    /// Жизнь из JSON старта; лобби здесь — ошибка теста.
    fn life(start_json: &str) -> Game {
        match start(start_json).expect("жизнь начинается") {
            Session::Life(game) => *game,
            Session::Lobby(_) => panic!("{start_json}: ожидалась жизнь, а открылось лобби"),
        }
    }

    fn parsed(json: &str) -> Value {
        serde_json::from_str(json).expect("мост отвечает валидным JSON")
    }

    /// Номер варианта команды; `match` без «остального» не соберётся, если в ядре появится
    /// команда, которой нет в этом списке.
    fn variant(command: &Command) -> usize {
        match command {
            Command::NewLife(_) => 0,
            Command::Advance { .. } => 1,
            Command::SetSpeed { .. } => 2,
            Command::Pause { .. } => 3,
            Command::FastForward { .. } => 4,
            Command::Buy { .. } => 5,
            Command::Sell { .. } => 6,
            Command::Travel { .. } => 7,
            Command::Choose { .. } => 8,
            Command::Continue => 9,
            Command::OpenScreen { .. } => 10,
            Command::OpenBusiness { .. } => 11,
            Command::SetPolicy { .. } => 12,
            Command::Hire { .. } => 13,
            Command::Fire { .. } => 14,
            Command::Call { .. } => 15,
            Command::ErasePagerMessage { .. } => 16,
            Command::BuyHousing { .. } => 17,
            Command::Retire { .. } => 18,
            Command::SetWork { .. } => 19,
            Command::UseService { .. } => 20,
            Command::OrderParcel => 21,
            Command::ApplyForParole => 22,
            Command::BribeForRelease => 23,
            Command::Deposit { .. } => 24,
            Command::Withdraw { .. } => 25,
            Command::Save { .. } => 26,
            Command::Load { .. } => 27,
            Command::Quit => 28,
        }
    }

    /// Чистый каталог игрока для теста.
    fn storage(test: &str) -> PathBuf {
        let directory = std::env::temp_dir()
            .join("speculant-godot-tests")
            .join(test);
        if directory.exists() {
            std::fs::remove_dir_all(&directory).expect("прошлый каталог теста стирается");
        }
        directory
    }

    /// JSON старта в каталоге игрока; `begin` — поля после `storage` или пусто для лобби.
    fn begin(storage: &std::path::Path, begin: &str) -> String {
        let storage = Value::from(storage.to_string_lossy());
        if begin.is_empty() {
            format!(r#"{{"storage":{storage}}}"#)
        } else {
            format!(r#"{{"storage":{storage},{begin}}}"#)
        }
    }

    #[test]
    fn every_command_converts_from_its_json() {
        let mut variants: Vec<usize> = COMMANDS
            .iter()
            .map(|json| {
                let command: Command =
                    serde_json::from_str(json).unwrap_or_else(|error| panic!("{json}: {error}"));
                variant(&command)
            })
            .collect();
        variants.sort_unstable();
        variants.dedup();
        assert_eq!(
            variants,
            (0..=28).collect::<Vec<_>>(),
            "у каждой команды есть JSON"
        );
    }

    #[test]
    fn every_command_answers_ok_or_a_json_error_with_a_message() {
        for json in COMMANDS {
            let mut game = life(LIFE);
            let answer = apply(&mut game, json);
            if answer == OK {
                continue;
            }
            let error = parsed(&answer);
            assert!(
                error["error"].is_string() && error["message"].is_string(),
                "{json}: {answer}"
            );
        }
    }

    #[test]
    fn a_new_life_from_settings_json_opens_the_market() {
        let game = life(LIFE);
        let view: StateView =
            serde_json::from_str(&view_json(&game)).expect("снимок разбирается обратно");
        assert_eq!(view.header.city, "Москва");
        assert_eq!(
            view.market.goods.len(),
            31,
            "из сорока товаров девять появятся позже 1984 года"
        );
        assert_eq!(view.header.date, "1 января 1984, вс");
    }

    #[test]
    fn malformed_settings_name_the_problem() {
        let error = parsed(&start("{не json").expect_err("битый JSON отвергнут"));
        assert_eq!(error["error"], "malformed_settings");
        let unknown = parsed(&start(r#"{"replay":"first"}"#).expect_err("старта нет"));
        assert_eq!(unknown["error"], "malformed_settings");
        let both = parsed(
            &start(r#"{"life":{"biography":"student","ironman":false,"seed":7},"resume":"first"}"#)
                .expect_err("жизнь и слот разом"),
        );
        assert_eq!(both["error"], "malformed_settings");
    }

    #[test]
    fn core_refusals_carry_their_kind_and_a_message() {
        let error = parsed(
            &start(r#"{"life":{"biography":"","ironman":false,"seed":1}}"#)
                .expect_err("пустая биография отвергнута"),
        );
        assert_eq!(error["error"], "empty_biography");
        assert_eq!(error["message"], "жизнь не начинается без биографии");
    }

    #[test]
    fn apply_runs_a_command_and_answers_ok() {
        let mut game = life(LIFE);
        assert_eq!(apply(&mut game, r#"{"command":"set_speed","speed":2}"#), OK);
        assert_eq!(game.view().time.speed, 2);
        assert_eq!(
            parsed(&apply(&mut game, r#"{"command":"fly"}"#))["error"],
            "malformed_command"
        );
    }

    #[test]
    fn buying_through_the_bridge_fills_the_pocket() {
        let mut game = life(LIFE);
        assert_eq!(
            apply(
                &mut game,
                r#"{"command":"buy","good":"chewing_gum","quantity":2}"#
            ),
            OK
        );
        assert_eq!(game.view().pocket.items[0].amount, "2 бл");
        let refusal = parsed(&apply(
            &mut game,
            r#"{"command":"buy","good":"jeans","quantity":1}"#,
        ));
        assert_eq!(refusal["error"], "not_enough_cash");
        assert!(
            refusal["message"]
                .as_str()
                .is_some_and(|message| message.starts_with("не хватает денег")),
            "{refusal}"
        );
    }

    #[test]
    fn advance_tells_whether_the_world_changed_and_refuses_a_negative_step() {
        let mut game = life(LIFE);
        assert_eq!(advance(&mut game, 1800), Ok(true));
        assert_eq!(game.view().header.date, "2 января 1984, пн");
        assert_eq!(advance(&mut game, 0), Ok(false), "без дня мир тот же");
        assert_eq!(
            parsed(&advance(&mut game, -1).expect_err("отрицательный шаг отвергнут"))["error"],
            "invalid_delta"
        );
    }

    #[test]
    fn a_life_without_a_directory_has_nowhere_to_save() {
        let mut game = life(r#"{"life":{"biography":"student","ironman":false,"seed":7}}"#);
        assert_eq!(
            parsed(&apply(&mut game, r#"{"command":"save","slot":"first"}"#))["error"],
            "no_storage"
        );
        assert_eq!(
            parsed(&start(r#"{"resume":"first"}"#).expect_err("слота без каталога нет"))["error"],
            "no_storage"
        );
        assert_eq!(
            parsed(&start("{}").expect_err("лобби без каталога нет"))["error"],
            "no_storage"
        );
    }

    #[test]
    fn a_life_saved_through_the_bridge_loads_and_resumes_with_the_same_view() {
        let directory = storage("saved_life");
        let mut game = life(&begin(
            &directory,
            r#""life":{"biography":"student","ironman":false,"seed":7}"#,
        ));
        assert_eq!(advance(&mut game, 1800), Ok(true));
        assert_eq!(apply(&mut game, r#"{"command":"save","slot":"first"}"#), OK);
        let saved = view_json(&game);
        assert!(
            directory.join("saves/first.spk").is_file(),
            "ядро пишет слот само"
        );
        assert_eq!(
            apply(
                &mut game,
                r#"{"command":"buy","good":"chewing_gum","quantity":1}"#
            ),
            OK
        );
        assert_ne!(view_json(&game), saved);
        assert_eq!(apply(&mut game, r#"{"command":"load","slot":"first"}"#), OK);
        assert_eq!(view_json(&game), saved);
        let resumed = life(&begin(&directory, r#""resume":"first""#));
        assert_eq!(view_json(&resumed), saved);
    }

    #[test]
    fn an_ironman_life_lies_in_its_slot_from_birth() {
        let directory = storage("ironman_life");
        let game = life(&begin(
            &directory,
            r#""life":{"biography":"student","ironman":true,"seed":7}"#,
        ));
        let resumed = life(&begin(&directory, r#""resume":"ironman""#));
        assert_eq!(view_json(&resumed), view_json(&game));
    }

    #[test]
    fn a_lobby_opens_the_player_directory_without_a_life() {
        let directory = storage("lobby");
        let Session::Lobby(lobby) = start(&begin(&directory, "")).expect("лобби открывается")
        else {
            panic!("без жизни и слота открывается лобби");
        };
        let view: LobbyView =
            serde_json::from_str(&lobby_view_json(&lobby)).expect("снимок лобби разбирается");
        let student = view
            .biographies
            .iter()
            .find(|biography| biography.biography == "student")
            .expect("студент в данных");
        assert!(student.unlocked, "стартовая биография открыта с начала");
        assert!(view.notebook.is_empty() && view.records.ironman.is_empty());
        assert!(!directory.join("saves").exists(), "лобби ничего не пишет");
    }

    #[test]
    fn no_life_is_an_error_kind_of_its_own() {
        assert_eq!(parsed(&no_life())["error"], "no_life");
    }
}
