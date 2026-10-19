use std::fs;

use anyhow::{Context, Result, bail};

use crate::export::{self, DIST, Platform};
use crate::godot_check::{godot_errors, run_godot};
use crate::godot_config::GodotConfig;
use crate::project_root::project_root;

pub const PRESETS_FILE: &str = "godot/export_presets.cfg";
/// Инструменты разработки в игру не попадают.
const TOOLS_FILTER: &str = "tools/*";
/// Без консоли (Windows) вывод игры виден только в файле лога.
const CHECK_LOG: &str = "check.log";

/// `cargo xtask export-check`: пресеты трёх платформ валидны, экспорт хост-платформы собирается
/// пином Godot и запускается `--headless --quit` без ошибок — мост грузится, заставка открывает
/// лобби.
pub fn run() -> Result<()> {
    let path = project_root().join(PRESETS_FILE);
    let text = fs::read_to_string(&path).with_context(|| format!("нет {}", path.display()))?;
    check_presets(&GodotConfig::parse(&text))?;
    let platform = Platform::host()?;
    let export_dir = export::export(platform)?;
    let log = project_root()
        .join(DIST)
        .join(format!("{}.{CHECK_LOG}", platform.name()));
    let log_path = log.to_string_lossy();
    run_godot(
        &platform.executable(&export_dir)?,
        &["--headless", "--quit", "--log-file", &log_path],
    )?;
    let log_text =
        fs::read_to_string(&log).with_context(|| format!("игра не записала {}", log.display()))?;
    let errors = godot_errors(&log_text);
    if !errors.is_empty() {
        bail!(
            "экспорт {} запустился с ошибками:\n{}",
            platform.name(),
            errors.join("\n")
        );
    }
    println!(
        "export-check: пресеты трёх платформ валидны, экспорт {} собирается и запускается",
        platform.name()
    );
    Ok(())
}

/// Ровно три пресета — по одному на платформу релиза — с её платформой Godot и архитектурой;
/// Windows и Linux вшивают pck в исполняемый файл; инструменты разработки исключены.
fn check_presets(config: &GodotConfig) -> Result<()> {
    let presets = config.presets();
    if presets.len() != Platform::ALL.len() {
        bail!(
            "{PRESETS_FILE}: пресетов {}, а платформ релиза {}",
            presets.len(),
            Platform::ALL.len()
        );
    }
    for platform in Platform::ALL {
        let preset = presets
            .iter()
            .find(|section| config.value(section, "name") == Some(platform.preset()))
            .with_context(|| format!("{PRESETS_FILE}: нет пресета «{}»", platform.preset()))?;
        let options = format!("{preset}.options");
        let expect = |section: &str, key: &str, expected: &str| -> Result<()> {
            let actual = config.value(section, key).unwrap_or_default();
            if actual != expected {
                bail!(
                    "{PRESETS_FILE}: у «{}» {key} = «{actual}», нужно «{expected}»",
                    platform.preset()
                );
            }
            Ok(())
        };
        expect(preset, "platform", platform.godot_platform())?;
        expect(preset, "exclude_filter", TOOLS_FILTER)?;
        expect(
            &options,
            "binary_format/architecture",
            platform.architecture(),
        )?;
        if platform != Platform::MacosUniversal {
            expect(&options, "binary_format/embed_pck", "true")?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::check_presets;
    use crate::godot_config::GodotConfig;

    const PRESETS: &str = include_str!("../../../godot/export_presets.cfg");

    #[test]
    fn the_committed_presets_cover_the_three_release_platforms() {
        check_presets(&GodotConfig::parse(PRESETS)).expect("пресеты проекта валидны");
    }

    #[test]
    fn a_wrong_architecture_or_a_missing_preset_is_refused() {
        let arm = PRESETS.replacen(
            "binary_format/architecture=\"universal\"",
            "binary_format/architecture=\"arm64\"",
            1,
        );
        let error = check_presets(&GodotConfig::parse(&arm)).expect_err("arm64 вместо universal");
        assert!(error.to_string().contains("macOS universal"), "{error}");
        let without_linux = PRESETS.replacen("name=\"Linux x86_64\"", "name=\"Linux arm64\"", 1);
        let error = check_presets(&GodotConfig::parse(&without_linux)).expect_err("нет Linux");
        assert!(
            error.to_string().contains("нет пресета «Linux x86_64»"),
            "{error}"
        );
    }
}
