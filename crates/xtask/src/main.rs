//! Сквозные команды проекта. Запуск: `cargo xtask <команда>`.
//! Каждая команда — один argv без shell, чтобы служить критерием готовности фазы.

mod build_godot;
mod charset_check;
mod deps_check;
mod docs_check;
mod etalons;
mod font_glyphs;
mod gd_check;
mod godot_check;
mod godot_pin;
mod project_root;
mod screen_check;
mod screen_file;
mod screen_runner;
mod tools_manifest;

use anyhow::{Result, bail};

const USAGE: &str = "\
Использование: cargo xtask <команда>

  godot-fetch               скачать пин Godot из tools.toml в .tools/ (проверка sha256)
  build-godot               собрать cdylib моста для хоста в godot/bin/
  godot-check               безголовый импорт проекта godot/ пином Godot и собранным мостом
  font-glyphs               записать godot/assets/fonts/glyphs.txt из TTF шрифта игры
  gd-check                  проверить парсером Godot все скрипты godot/src и godot/tools
  charset-check             строки godot/src/**/*.gd и data/** только из глифов шрифта
  etalons [--check]         снять эталоны шести макетов и палитру из дизайн-книги (Boa)
  screen-check <экран>      буфер экрана Godot против design/etalons/<экран>.screen
  screenshot <экран>        PNG экрана в .tools/out/ (оконный запуск)
  deps-check                ядро speculant-core без Godot; пин gdext согласован с tools.toml
  docs-check [--all-decided] маркеры DECIDED-15-N совпадают в документе и дизайн-книге";

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = arguments.first() else {
        bail!("{USAGE}");
    };
    let flags = &arguments[1..];
    match command.as_str() {
        "godot-fetch" => {
            let binary = godot_pin::fetch()?;
            println!("Godot: {}", binary.display());
            Ok(())
        }
        "build-godot" => {
            let library = build_godot::build_host_debug()?;
            println!("Мост: {}", library.display());
            Ok(())
        }
        "godot-check" => godot_check::run(),
        "font-glyphs" => font_glyphs::run(),
        "gd-check" => gd_check::run(),
        "charset-check" => charset_check::run(),
        "etalons" => etalons::run(flags.iter().any(|flag| flag == "--check")),
        "screen-check" => screen_check::run(single_argument(flags, "screen-check <экран>")?),
        "screenshot" => {
            let png = screen_runner::screenshot(single_argument(flags, "screenshot <экран>")?)?;
            println!("screenshot: {}", png.display());
            Ok(())
        }
        "deps-check" => deps_check::run(),
        "docs-check" => docs_check::run(flags.iter().any(|flag| flag == "--all-decided")),
        unknown => bail!("неизвестная команда xtask: {unknown}\n{USAGE}"),
    }
}

fn single_argument<'a>(flags: &'a [String], usage: &str) -> Result<&'a str> {
    match flags {
        [argument] => Ok(argument.as_str()),
        _ => bail!("использование: cargo xtask {usage}"),
    }
}
