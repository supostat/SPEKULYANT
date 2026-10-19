use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result, anyhow, bail};
use speculant_core::content::EventId;
use speculant_core::save::Storage;
use speculant_core::{Command, Content, Game, LifeSettings, Lobby};

use crate::project_root::project_root;
use crate::screen_file::{COLUMNS, ROWS, ScreenFile};
use crate::screen_runner;

/// Все экраны без макета и снимок, на котором их рисовать.
const SCREENS: [(&str, Snapshot); 15] = [
    ("main_menu", Snapshot::Lobby),
    ("settings", Snapshot::Lobby),
    ("new_life", Snapshot::Lobby),
    ("records", Snapshot::Lobby),
    ("notebook", Snapshot::Lobby),
    ("slots", Snapshot::Life),
    ("quit", Snapshot::Life),
    ("help", Snapshot::Life),
    ("game_menu", Snapshot::Life),
    ("travel", Snapshot::Life),
    ("businesses", Snapshot::Life),
    ("hire", Snapshot::Life),
    ("contacts", Snapshot::Life),
    ("money", Snapshot::Life),
    ("zone", Snapshot::Zone),
];
const ALL: &str = "all";
/// Снимок жизни — студент с этим сидом на настоящих данных.
const BIOGRAPHY: &str = "student";
const SEED: u64 = 7;
/// В зону — моряк: денег хватает на сигнал в ОБХСС, а проверка кончается судом, если отпираться.
const ZONE_BIOGRAPHY: &str = "sailor";
const ZONE_CHAIN: [(&str, usize); 2] = [("obkhss_signal", 3), ("obkhss_visit", 3)];
/// Лобби — каталог игрока, где прошлых жизней не было: лобби в нём ничего не пишет.
const LOBBY_STORAGE: &str = ".tools/out/screen-smoke-lobby";
const FKEY_ROW: usize = ROWS - 1;
const KEY_COLORS: (char, char) = ('f', '0');
const LABEL_COLORS: (char, char) = ('0', '3');

#[derive(Clone, Copy)]
enum Snapshot {
    Lobby,
    Life,
    Zone,
}

/// `cargo xtask screen-smoke <экран|all>`: экран без эталона рисуется безголово на снимке
/// настоящего ядра без ошибок Godot, и строка F-клавиш стоит на месте — с клавиши 1, номера белым
/// на чёрном, подписи чёрным на бирюзовом во всю ширину. `all` — все экраны без макета.
pub fn run(screen: &str) -> Result<()> {
    let screens: Vec<(&str, Snapshot)> = if screen == ALL {
        SCREENS.to_vec()
    } else {
        let known = SCREENS
            .iter()
            .find(|(name, _)| *name == screen)
            .with_context(|| {
                format!(
                    "у экрана «{screen}» есть макет или его нет вовсе; без макета: {}",
                    names()
                )
            })?;
        vec![*known]
    };
    let content = content()?;
    for (name, snapshot) in screens {
        let view = write_view(name, snapshot, &content)?;
        let dumped = screen_runner::dump_with(name, Some(&view))?;
        check_fkeys(&dumped).with_context(|| format!("экран {name}"))?;
        println!("screen-smoke {name}: экран рисуется на снимке ядра, строка F-клавиш на месте");
    }
    Ok(())
}

/// Снимок настоящего ядра для экрана без макета в `.tools/out/<экран>.live.view.json` — на нём
/// экран рисуют smoke и `cargo xtask screenshot`.
pub fn live_view(screen: &str) -> Result<PathBuf> {
    let (name, snapshot) = SCREENS
        .iter()
        .find(|(name, _)| *name == screen)
        .with_context(|| format!("у экрана «{screen}» нет живого снимка: {}", names()))?;
    write_view(name, *snapshot, &content()?)
}

fn names() -> String {
    let names: Vec<&str> = SCREENS.iter().map(|(name, _)| *name).collect();
    names.join(", ")
}

fn content() -> Result<Arc<Content>> {
    let content = Content::load(&project_root().join("data")).map_err(|errors| {
        anyhow!(
            "data/ не загружается ({} ошибок) — сначала cargo run -p speculant-validate -- data",
            errors.len()
        )
    })?;
    Ok(Arc::new(content))
}

fn write_view(screen: &str, snapshot: Snapshot, content: &Arc<Content>) -> Result<PathBuf> {
    let view = screen_runner::output_path(screen, "live.view.json")?;
    fs::write(&view, snapshot_json(snapshot, content)?)?;
    Ok(view)
}

fn snapshot_json(snapshot: Snapshot, content: &Arc<Content>) -> Result<String> {
    let json = match snapshot {
        Snapshot::Lobby => {
            let directory = project_root().join(LOBBY_STORAGE);
            let lobby = Lobby::open(Arc::clone(content), &Storage::new(directory))
                .map_err(|error| anyhow!("лобби не открывается: {error}"))?;
            serde_json::to_string(&lobby.view())?
        }
        Snapshot::Life => serde_json::to_string(&life(BIOGRAPHY, content)?.view())?,
        Snapshot::Zone => serde_json::to_string(&zone_life(content)?.view())?,
    };
    Ok(json)
}

fn life(biography: &str, content: &Arc<Content>) -> Result<Game> {
    let settings = LifeSettings {
        biography: biography.to_owned(),
        ironman: false,
        seed: SEED,
    };
    Game::new_life(&settings, Arc::clone(content))
        .map_err(|error| anyhow!("жизнь не начинается: {error}"))
}

/// Жизнь, которую цепочка событий привела в зону.
fn zone_life(content: &Arc<Content>) -> Result<Game> {
    let mut game = life(ZONE_BIOGRAPHY, content)?;
    for (event, choice) in ZONE_CHAIN {
        game.start_event(&EventId::new(event))
            .map_err(|error| anyhow!("событие {event} не начинается: {error}"))?;
        for command in [Command::Choose { choice }, Command::Continue] {
            game.apply(command)
                .map_err(|error| anyhow!("событие {event}: {error}"))?;
        }
    }
    if game.zone().is_none() {
        bail!("цепочка {ZONE_CHAIN:?} не привела в зону");
    }
    Ok(game)
}

fn check_fkeys(screen: &ScreenFile) -> Result<()> {
    let characters: Vec<char> = screen.characters[FKEY_ROW].chars().collect();
    let foreground: Vec<char> = screen.foreground[FKEY_ROW].chars().collect();
    let background: Vec<char> = screen.background[FKEY_ROW].chars().collect();
    if characters[0] != '1' {
        bail!(
            "строка F-клавиш начинается с «{}», а не с клавиши 1",
            characters[0]
        );
    }
    for column in 0..COLUMNS {
        let colors = (foreground[column], background[column]);
        if colors != KEY_COLORS && colors != LABEL_COLORS {
            bail!("строка F-клавиш, колонка {column}: цвета {colors:?} — ни клавиша, ни подпись");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{FKEY_ROW, check_fkeys};
    use crate::screen_file::{COLUMNS, ROWS, ScreenFile};

    /// Экран со строкой F-клавиш поездки: «1» белым на чёрном, подписи чёрным на бирюзовом.
    fn screen_with_fkeys() -> ScreenFile {
        let mut screen = ScreenFile {
            characters: vec![" ".repeat(COLUMNS); ROWS],
            foreground: vec!["7".repeat(COLUMNS); ROWS],
            background: vec!["1".repeat(COLUMNS); ROWS],
        };
        screen.characters[FKEY_ROW] = format!("{:<80}", "1Помощь  2        10Назад");
        screen.foreground[FKEY_ROW] = format!("f{}", "0".repeat(COLUMNS - 1));
        screen.background[FKEY_ROW] = format!("0{}", "3".repeat(COLUMNS - 1));
        screen
    }

    #[test]
    fn a_full_row_of_keys_and_labels_passes() {
        assert!(check_fkeys(&screen_with_fkeys()).is_ok());
    }

    #[test]
    fn a_row_without_key_one_or_with_window_colors_fails() {
        let mut shifted = screen_with_fkeys();
        shifted.characters[FKEY_ROW] = format!("{:<80}", " Помощь");
        assert!(check_fkeys(&shifted).is_err());
        let mut covered = screen_with_fkeys();
        covered.background[FKEY_ROW] = format!("0{}1", "3".repeat(COLUMNS - 2));
        assert!(
            check_fkeys(&covered)
                .expect_err("колонка цвета окна")
                .to_string()
                .contains("колонка 79")
        );
    }
}
