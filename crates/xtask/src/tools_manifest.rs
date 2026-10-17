use std::collections::BTreeMap;
use std::fs;

use anyhow::{Context, Result, bail};
use serde::Deserialize;

use crate::project_root::project_root;

/// Пин инструментов проекта из `tools.toml`.
#[derive(Deserialize)]
pub struct ToolsManifest {
    pub godot: GodotPin,
    pub gdext: GdextPin,
}

#[derive(Deserialize)]
pub struct GodotPin {
    pub version: String,
    pub release_base_url: String,
    pub archives: BTreeMap<String, GodotArchive>,
}

#[derive(Deserialize)]
pub struct GodotArchive {
    pub file: String,
    pub sha256: String,
    pub binary: String,
}

#[derive(Deserialize)]
pub struct GdextPin {
    pub version: String,
    pub api_feature: String,
}

pub fn load() -> Result<ToolsManifest> {
    let path = project_root().join("tools.toml");
    let text =
        fs::read_to_string(&path).with_context(|| format!("не читается {}", path.display()))?;
    toml::from_str(&text)
        .with_context(|| format!("tools.toml: ошибка формата в {}", path.display()))
}

/// Имя цели хост-машины — ключ таблицы `[godot.archives]`.
pub fn host_target() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", _) => Ok("darwin-universal"),
        ("linux", "x86_64") => Ok("linux-x86_64"),
        ("linux", "aarch64") => Ok("linux-aarch64"),
        ("windows", "x86_64") => Ok("windows-x86_64"),
        (os, arch) => bail!("платформа {os}/{arch} не входит в пин Godot (tools.toml)"),
    }
}
