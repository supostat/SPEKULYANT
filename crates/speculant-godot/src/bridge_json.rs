//! Конвертации моста без Godot: JSON интерфейса ↔ типы ядра. Отделены от класса Godot,
//! чтобы проверяться обычными тестами.
//!
//! Ответ на команду — `{"ok":true}` или `{"error":"<вид>", ..., "message":"<текст для игрока>"}`.

use serde_json::{Value, json};
use speculant_core::{Command, CommandError, Game, LifeSettings};

pub const OK: &str = r#"{"ok":true}"#;

pub fn new_life(settings_json: &str) -> Result<Game, String> {
    let settings: LifeSettings = serde_json::from_str(settings_json)
        .map_err(|error| bridge_error("malformed_settings", error.to_string()))?;
    Game::new_life(&settings).map_err(|error| command_error(&error))
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
    serde_json::to_string(game.view()).expect("StateView сериализуется в JSON")
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
        let refusal = parsed(&apply(
            &mut game,
            r#"{"command":"buy","good":"jeans","quantity":1}"#,
        ));
        assert_eq!(refusal["error"], "not_simulated_yet");
        assert_eq!(refusal["command"], "buy");
    }

    #[test]
    fn advance_refuses_a_negative_step() {
        let mut game = new_life(SETTINGS).expect("жизнь начинается");
        assert_eq!(advance(&mut game, 1800), Ok(false));
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
