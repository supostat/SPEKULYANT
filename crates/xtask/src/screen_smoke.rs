use std::fs;
use std::sync::Arc;

use anyhow::{Result, anyhow, bail};
use speculant_core::{Content, Game, LifeSettings};

use crate::project_root::project_root;
use crate::screen_file::{COLUMNS, ROWS, ScreenFile};
use crate::screen_runner;

/// Снимок для экрана — новая жизнь студента с этим сидом на настоящих данных.
const BIOGRAPHY: &str = "student";
const SEED: u64 = 7;
const FKEY_ROW: usize = ROWS - 1;
const KEY_COLORS: (char, char) = ('f', '0');
const LABEL_COLORS: (char, char) = ('0', '3');

/// `cargo xtask screen-smoke <экран>`: экран без эталона рисуется безголово на снимке настоящего
/// ядра без ошибок Godot, и строка F-клавиш стоит на месте — с клавиши 1, номера белым на чёрном,
/// подписи чёрным на бирюзовом во всю ширину.
pub fn run(screen: &str) -> Result<()> {
    let data = project_root().join("data");
    let content = Content::load(&data).map_err(|errors| {
        anyhow!(
            "data/ не загружается ({} ошибок) — сначала cargo run -p speculant-validate -- data",
            errors.len()
        )
    })?;
    let settings = LifeSettings {
        biography: BIOGRAPHY.to_owned(),
        ironman: false,
        seed: SEED,
    };
    let game = Game::new_life(&settings, Arc::new(content))
        .map_err(|error| anyhow!("жизнь не начинается: {error}"))?;
    let view = screen_runner::output_path(screen, "live.view.json")?;
    fs::write(&view, serde_json::to_string(&game.view())?)?;
    let dumped = screen_runner::dump_with(screen, Some(&view))?;
    check_fkeys(&dumped)?;
    println!("screen-smoke {screen}: экран рисуется на снимке ядра, строка F-клавиш на месте");
    Ok(())
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
