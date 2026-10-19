//! Класс `SpeculantGame` для GDScript: четыре метода, всё остальное — в `bridge_json` и ядре.

use godot::prelude::*;

use crate::bridge_json::{self, Session};

#[derive(GodotClass)]
#[class(init, base=RefCounted)]
pub struct SpeculantGame {
    session: Option<Session>,
}

#[godot_api]
impl SpeculantGame {
    /// Начинает сеанс: `{"storage": <каталог игрока>, "life": {"biography", "ironman", "seed"}}`,
    /// `{"storage": <каталог игрока>, "resume": <слот>}` или `{"storage": <каталог игрока>}` —
    /// лобби без жизни; отвечает `{"ok":true}` или ошибкой, и прежний сеанс тогда остаётся.
    #[func]
    fn new_life(&mut self, start_json: GString) -> GString {
        match bridge_json::start(&start_json.to_string()) {
            Ok(session) => {
                self.session = Some(session);
                GString::from(bridge_json::OK)
            }
            Err(error) => GString::from(&error),
        }
    }

    /// Двигает мир на `delta_ms` реального времени; `true`, если снимок изменился. В лобби
    /// времени нет.
    #[func]
    fn advance(&mut self, delta_ms: i64) -> bool {
        let Some(Session::Life(game)) = self.session.as_mut() else {
            godot_error!("{}", bridge_json::no_life());
            return false;
        };
        bridge_json::advance(game, delta_ms).unwrap_or_else(|error| {
            godot_error!("{error}");
            false
        })
    }

    /// Применяет команду `{"command": "<имя>", ...}`; отвечает `{"ok":true}` или ошибкой. В лобби
    /// команд нет — жизнь начинается через `new_life`.
    #[func]
    fn apply(&mut self, command_json: GString) -> GString {
        let answer = match self.session.as_mut() {
            Some(Session::Life(game)) => bridge_json::apply(game, &command_json.to_string()),
            Some(Session::Lobby(_)) | None => bridge_json::no_life(),
        };
        GString::from(&answer)
    }

    /// Снимок JSON-строкой: `StateView` идущей жизни или `LobbyView` лобби.
    #[func]
    fn view_json(&self) -> GString {
        let snapshot = match &self.session {
            Some(Session::Life(game)) => bridge_json::view_json(game),
            Some(Session::Lobby(lobby)) => bridge_json::lobby_view_json(lobby),
            None => bridge_json::no_life(),
        };
        GString::from(&snapshot)
    }
}
