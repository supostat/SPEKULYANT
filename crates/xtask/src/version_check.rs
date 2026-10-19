use std::fs;
use std::process::Command;

use anyhow::{Context, Result, bail};

use crate::export_check::PRESETS_FILE;
use crate::godot_config::GodotConfig;
use crate::project_root::project_root;

const PROJECT_FILE: &str = "godot/project.godot";
const PROJECT_SECTION: &str = "application";
const PROJECT_VERSION_KEY: &str = "config/version";
/// Опции пресетов, которыми экспорт переопределил бы версию проекта: они пусты — версия одна.
const PRESET_VERSION_KEYS: [&str; 4] = [
    "application/file_version",
    "application/product_version",
    "application/short_version",
    "application/version",
];
const TAG_PREFIX: &str = "v";

/// `cargo xtask version-check`: версия игры — одна, `[workspace.package] version` в `Cargo.toml`;
/// `project.godot` повторяет её в `application/config/version`, пресеты экспорта её не
/// переопределяют, а тег релиза на этом коммите — `v<версия>`. Тег берётся из `GITHUB_REF_NAME`,
/// если сборка идёт по тегу, иначе — из `git tag --points-at HEAD`.
pub fn run() -> Result<()> {
    let version = env!("CARGO_PKG_VERSION");
    let project = GodotConfig::parse(&read(PROJECT_FILE)?);
    check_project(&project, version)?;
    check_presets(&GodotConfig::parse(&read(PRESETS_FILE)?))?;
    let tags = release_tags()?;
    check_tags(&tags, version)?;
    let tag = tags.first().map_or("тега нет", String::as_str);
    println!(
        "version-check: версия {version} — Cargo.toml, {PROJECT_FILE}, пресеты её наследуют; {tag}"
    );
    Ok(())
}

fn read(file: &str) -> Result<String> {
    let path = project_root().join(file);
    fs::read_to_string(&path).with_context(|| format!("нет {}", path.display()))
}

fn check_project(project: &GodotConfig, version: &str) -> Result<()> {
    let declared = project
        .value(PROJECT_SECTION, PROJECT_VERSION_KEY)
        .with_context(|| format!("{PROJECT_FILE}: нет {PROJECT_VERSION_KEY}"))?;
    if declared != version {
        bail!("{PROJECT_FILE}: версия {declared}, а в Cargo.toml — {version}");
    }
    Ok(())
}

fn check_presets(presets: &GodotConfig) -> Result<()> {
    for preset in presets.presets() {
        let options = format!("{preset}.options");
        for key in PRESET_VERSION_KEYS {
            if let Some(value) = presets
                .value(&options, key)
                .filter(|value| !value.is_empty())
            {
                bail!(
                    "{PRESETS_FILE}: {preset} задаёт {key} = «{value}» — версия должна приходить из проекта"
                );
            }
        }
    }
    Ok(())
}

/// Теги релиза `v*` этого коммита.
fn release_tags() -> Result<Vec<String>> {
    if std::env::var("GITHUB_REF_TYPE").as_deref() == Ok("tag") {
        let tag = std::env::var("GITHUB_REF_NAME").context("сборка по тегу без GITHUB_REF_NAME")?;
        return Ok(vec![tag]);
    }
    let output = Command::new("git")
        .args([
            "tag",
            "--points-at",
            "HEAD",
            "--list",
            &format!("{TAG_PREFIX}*"),
        ])
        .current_dir(project_root())
        .output()
        .context("git не запускается")?;
    if !output.status.success() {
        bail!(
            "git tag завершился с кодом {}: {}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(String::from_utf8(output.stdout)?
        .lines()
        .map(str::to_owned)
        .collect())
}

fn check_tags(tags: &[String], version: &str) -> Result<()> {
    let expected = format!("{TAG_PREFIX}{version}");
    if let Some(tag) = tags.iter().find(|tag| **tag != expected) {
        bail!("тег {tag} не совпадает с версией {version}: нужен {expected}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{check_presets, check_project, check_tags};
    use crate::godot_config::GodotConfig;

    #[test]
    fn the_project_repeats_the_workspace_version() {
        let project = GodotConfig::parse("[application]\nconfig/version=\"0.1.0\"\n");
        assert!(check_project(&project, "0.1.0").is_ok());
        assert!(check_project(&project, "0.2.0").is_err());
        assert!(check_project(&GodotConfig::parse("[application]\n"), "0.1.0").is_err());
    }

    #[test]
    fn a_preset_that_overrides_the_version_is_refused() {
        let inherit =
            GodotConfig::parse("[preset.0]\n[preset.0.options]\napplication/version=\"\"\n");
        assert!(check_presets(&inherit).is_ok());
        let pinned = GodotConfig::parse(
            "[preset.0]\n[preset.0.options]\napplication/file_version=\"1.0.0\"\n",
        );
        assert!(check_presets(&pinned).is_err());
    }

    #[test]
    fn only_the_tag_of_the_version_is_accepted() {
        assert!(check_tags(&[], "0.1.0").is_ok());
        assert!(check_tags(&["v0.1.0".to_owned()], "0.1.0").is_ok());
        assert!(check_tags(&["v0.1.1".to_owned()], "0.1.0").is_err());
    }
}
