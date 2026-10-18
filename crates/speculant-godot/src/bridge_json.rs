//! Конвертации моста без Godot: JSON интерфейса ↔ типы ядра. Отделены от класса Godot,
//! чтобы проверяться обычными тестами.
//!
//! Ответ на команду — `{"ok":true}` или `{"error":"<вид>", ..., "message":"<текст для игрока>"}`.

use serde_json::{Value, json};
use std::path::Path;
use std::sync::{Arc, OnceLock};

use include_dir::{Dir, include_dir};
use speculant_core::content::{ContentError, ContentFile};
use speculant_core::{Command, CommandError, Content, Game, LifeSettings};

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

pub fn new_life(settings_json: &str) -> Result<Game, String> {
    let settings: LifeSettings = serde_json::from_str(settings_json)
        .map_err(|error| bridge_error("malformed_settings", error.to_string()))?;
    Game::new_life(&settings, content()?).map_err(|error| command_error(&error))
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
    use serde_json::Value;
    use speculant_core::StateView;

    use super::{OK, advance, apply, new_life, no_life, view_json};

    const SETTINGS: &str = r#"{"biography":"student","ironman":true,"seed":7}"#;

    fn parsed(json: &str) -> Value {
        serde_json::from_str(json).expect("мост отвечает валидным JSON")
    }

    #[test]
    fn new_life_from_settings_json_opens_the_market() {
        let game = new_life(SETTINGS).expect("жизнь начинается");
        let view: StateView =
            serde_json::from_str(&view_json(&game)).expect("снимок разбирается обратно");
        assert_eq!(view.header.city, "Москва");
        assert_eq!(view.market.goods.len(), 8);
        assert_eq!(view.header.date, "1 января 1984, вс");
    }

    #[test]
    fn malformed_settings_name_the_problem() {
        let error = parsed(&new_life("{не json").expect_err("битый JSON отвергнут"));
        assert_eq!(error["error"], "malformed_settings");
    }

    #[test]
    fn core_refusals_carry_their_kind_and_a_message() {
        let error = parsed(
            &new_life(r#"{"biography":"","ironman":false,"seed":1}"#)
                .expect_err("пустая биография отвергнута"),
        );
        assert_eq!(error["error"], "empty_biography");
        assert_eq!(error["message"], "жизнь не начинается без биографии");
    }

    #[test]
    fn apply_runs_a_command_and_answers_ok() {
        let mut game = new_life(SETTINGS).expect("жизнь начинается");
        assert_eq!(apply(&mut game, r#"{"command":"set_speed","speed":2}"#), OK);
        assert_eq!(game.view().time.speed, 2);
    }

    #[test]
    fn apply_refuses_unknown_and_unsimulated_commands_as_json() {
        let mut game = new_life(SETTINGS).expect("жизнь начинается");
        assert_eq!(
            parsed(&apply(&mut game, r#"{"command":"fly"}"#))["error"],
            "malformed_command"
        );
        let refusal = parsed(&apply(&mut game, r#"{"command":"call","contact":"kofi"}"#));
        assert_eq!(refusal["error"], "not_simulated_yet");
        assert_eq!(refusal["command"], "call");
    }

    #[test]
    fn buying_through_the_bridge_fills_the_pocket() {
        let mut game = new_life(SETTINGS).expect("жизнь начинается");
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
    fn advance_refuses_a_negative_step() {
        let mut game = new_life(SETTINGS).expect("жизнь начинается");
        assert_eq!(advance(&mut game, 1800), Ok(true));
        assert_eq!(game.view().header.date, "2 января 1984, пн");
        assert_eq!(
            parsed(&advance(&mut game, -1).expect_err("отрицательный шаг отвергнут"))["error"],
            "invalid_delta"
        );
    }

    #[test]
    fn no_life_is_an_error_kind_of_its_own() {
        assert_eq!(parsed(&no_life())["error"], "no_life");
    }
}
