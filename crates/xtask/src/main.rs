//! Сквозные команды проекта. Запуск: `cargo xtask <команда>`.
//! Каждая команда — один argv без shell, чтобы служить критерием готовности фазы.

mod build_godot;
mod deps_check;
mod docs_check;
mod gd_check;
mod godot_check;
mod godot_pin;
mod project_root;
mod tools_manifest;

use anyhow::{Result, bail};

const USAGE: &str = "\
Использование: cargo xtask <команда>

  godot-fetch               скачать пин Godot из tools.toml в .tools/ (проверка sha256)
  build-godot               собрать cdylib моста для хоста в godot/bin/
  godot-check               безголовый импорт проекта godot/ пином Godot и собранным мостом
  gd-check                  проверить парсером Godot все скрипты godot/src и godot/tools
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
        "gd-check" => gd_check::run(),
        "deps-check" => deps_check::run(),
        "docs-check" => docs_check::run(flags.iter().any(|flag| flag == "--all-decided")),
        unknown => bail!("неизвестная команда xtask: {unknown}\n{USAGE}"),
    }
}
