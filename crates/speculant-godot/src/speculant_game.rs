//! Класс `SpeculantGame` для GDScript: четыре метода, всё остальное — в `bridge_json` и ядре.

use godot::prelude::*;
use speculant_core::Game;

use crate::bridge_json;

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SpeculantGame {
    game: Option<Game>,
}

#[godot_api]
impl SpeculantGame {
    /// Начинает жизнь: `{"storage": <каталог игрока>, "life": {"biography", "ironman", "seed"}}`
    /// или `{"storage": <каталог игрока>, "resume": <слот>}`; отвечает `{"ok":true}` или ошибкой.
    #[func]
    fn new_life(&mut self, start_json: GString) -> GString {
        match bridge_json::new_life(&start_json.to_string()) {
            Ok(game) => {
                self.game = Some(game);
                GString::from(bridge_json::OK)
            }
            Err(error) => GString::from(&error),
        }
    }

    /// Двигает мир на `delta_ms` реального времени; `true`, если снимок изменился.
    #[func]
    fn advance(&mut self, delta_ms: i64) -> bool {
        let Some(game) = self.game.as_mut() else {
            godot_error!("{}", bridge_json::no_life());
            return false;
        };
        bridge_json::advance(game, delta_ms).unwrap_or_else(|error| {
            godot_error!("{error}");
            false
        })
    }

    /// Применяет команду `{"command": "<имя>", ...}`; отвечает `{"ok":true}` или ошибкой.
    #[func]
    fn apply(&mut self, command_json: GString) -> GString {
        let answer = match self.game.as_mut() {
            Some(game) => bridge_json::apply(game, &command_json.to_string()),
            None => bridge_json::no_life(),
        };
        GString::from(&answer)
    }

    /// Снимок мира `StateView` JSON-строкой.
    #[func]
    fn view_json(&self) -> GString {
        let snapshot = match &self.game {
            Some(game) => bridge_json::view_json(game),
            None => bridge_json::no_life(),
        };
        GString::from(&snapshot)
    }
}
