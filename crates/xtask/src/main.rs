//! Сквозные команды проекта. Запуск: `cargo xtask <команда>`.
//! Каждая команда — один argv без shell, чтобы служить критерием готовности фазы.

mod balance_check;
mod bridge_smoke;
mod build_godot;
mod charset_check;
mod content_count;
mod coverage_check;
mod deps_check;
mod docs_check;
mod etalons;
mod font_glyphs;
mod gd_check;
mod godot_check;
mod godot_pin;
mod new_event;
mod project_root;
mod screen_check;
mod screen_file;
mod screen_runner;
mod screen_smoke;
mod sound_smoke;
mod tools_manifest;
mod ui_flow_smoke;

use anyhow::{Result, bail};

const USAGE: &str = "\
Использование: cargo xtask <команда>

  godot-fetch               скачать пин Godot из tools.toml в .tools/ (проверка sha256)
  build-godot [--release] [--universal]
                            собрать cdylib моста в godot/bin/ (universal — arm64+x86_64 на macOS)
  bridge-smoke [--full]     безголово создать SpeculantGame в Godot и прочитать снимок ядра;
                            --full — тридцать дней с событием, торговля, поездка, сохранения
                            и каждая команда интерфейса через мост
  godot-check               безголовый импорт проекта godot/ пином Godot и собранным мостом
  font-glyphs               записать godot/assets/fonts/glyphs.txt из TTF шрифта игры
  gd-check                  проверить парсером Godot все скрипты godot/src и godot/tools
  charset-check             строки godot/src/**/*.gd и data/** только из глифов шрифта
  etalons [--check]         снять эталоны шести макетов и палитру из дизайн-книги (Boa)
  screen-check <экран>      буфер экрана Godot против design/etalons/<экран>.screen
  screen-smoke <экран|all>  экран без макета рисуется на снимке ядра, строка F-клавиш на месте;
                            all — все экраны без макета
  screenshot <экран> [--crt] PNG экрана в .tools/out/ (оконный запуск): экран макета — на его
                            фикстуре, экран без макета — на снимке настоящего ядра; --crt —
                            с CRT-эффектом поверх, в <экран>.crt.png
  ui-flow-smoke             безголовый клавиатурный сценарий: заставка → меню → новая жизнь →
                            рынок, событие, поездка → дела, связи, пейджер → сохранение →
                            главное меню → загрузка → выход
  sound-smoke               безголово: пищалка PC-speaker выдаёт ноту заданной частоты, мелодия
                            заставки приходит из данных через мост
  deps-check                ядро speculant-core без Godot; пин gdext согласован с tools.toml
  new-event <эпоха> <имя>   заготовка события в data/events/<эпоха>/<имя>.toml
  docs-check [--all-decided] маркеры DECIDED-15-N совпадают в документе и дизайн-книге
  content-count <вид>       сколько в data/ товаров, городов, соперников (npc), событий
                            (events:<эпоха>, zone-events), шаблонов (post2004-templates)
                            — не меньше порога [volume] из data/balance.toml
  balance-check <эпоха|all> прогон [balance] lives жизней всеми стратегиями и сверка эпохи
                            или всех эпох с порогами [balance] из data/balance.toml
  coverage-check            прогон [coverage] lives жизней, строгая валидация data/ и сверка
                            покрытия: каждое событие сработало или помечено rare";

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
        "build-godot" => build_godot::run(flags),
        "bridge-smoke" => bridge_smoke::run(flags.iter().any(|flag| flag == "--full")),
        "godot-check" => godot_check::run(),
        "font-glyphs" => font_glyphs::run(),
        "gd-check" => gd_check::run(),
        "charset-check" => charset_check::run(),
        "etalons" => etalons::run(flags.iter().any(|flag| flag == "--check")),
        "screen-check" => screen_check::run(single_argument(flags, "screen-check <экран>")?),
        "screen-smoke" => screen_smoke::run(single_argument(flags, "screen-smoke <экран>")?),
        "ui-flow-smoke" => ui_flow_smoke::run(),
        "sound-smoke" => sound_smoke::run(),
        "screenshot" => {
            let (screen, crt) = match flags {
                [screen] => (screen.as_str(), false),
                [screen, flag] if flag == "--crt" => (screen.as_str(), true),
                _ => bail!("использование: cargo xtask screenshot <экран> [--crt]"),
            };
            let png = if etalons::SCREENS.contains(&screen) {
                screen_runner::screenshot(screen, crt)?
            } else {
                let view = screen_smoke::live_view(screen)?;
                screen_runner::screenshot_with(screen, Some(&view), crt)?
            };
            println!("screenshot: {}", png.display());
            Ok(())
        }
        "new-event" => new_event::run(flags),
        "deps-check" => deps_check::run(),
        "docs-check" => docs_check::run(flags.iter().any(|flag| flag == "--all-decided")),
        "content-count" => content_count::run(single_argument(flags, "content-count <вид>")?),
        "balance-check" => balance_check::run(single_argument(flags, "balance-check <эпоха|all>")?),
        "coverage-check" => coverage_check::run(),
        unknown => bail!("неизвестная команда xtask: {unknown}\n{USAGE}"),
    }
}

fn single_argument<'a>(flags: &'a [String], usage: &str) -> Result<&'a str> {
    match flags {
        [argument] => Ok(argument.as_str()),
        _ => bail!("использование: cargo xtask {usage}"),
    }
}
