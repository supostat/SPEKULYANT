use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;

use crate::project_root::project_root;

const BRIDGE_CRATE: &str = "speculant-godot";

/// Собирает cdylib моста для хост-платформы и кладёт его туда, куда смотрит `godot/speculant.gdextension`.
/// Релизная и universal-сборки появятся в фазе `bridge-contract`.
pub fn build_host_debug() -> Result<PathBuf> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let status = Command::new(cargo)
        .args(["build", "-p", BRIDGE_CRATE])
        .current_dir(project_root())
        .status()?;
    if !status.success() {
        bail!("cargo build -p {BRIDGE_CRATE} завершился с кодом {status}");
    }
    let target_dir = MetadataCommand::new()
        .manifest_path(project_root().join("Cargo.toml"))
        .no_deps()
        .exec()?
        .target_directory;
    let (platform_dir, library_file) = host_library_layout()?;
    let built = target_dir.join("debug").join(library_file);
    let destination_dir = project_root().join("godot").join("bin").join(platform_dir);
    fs::create_dir_all(&destination_dir)?;
    let destination = destination_dir.join(library_file);
    fs::copy(&built, &destination).with_context(|| format!("нет собранной библиотеки {built}"))?;
    Ok(destination)
}

/// Каталог в `godot/bin/` и имя файла библиотеки — ровно как в `speculant.gdextension`.
fn host_library_layout() -> Result<(String, &'static str)> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", _) => Ok(("macos".to_owned(), "libspeculant_godot.dylib")),
        ("linux", arch @ ("x86_64" | "aarch64")) => {
            Ok((format!("linux-{arch}"), "libspeculant_godot.so"))
        }
        ("windows", "x86_64") => Ok(("windows-x86_64".to_owned(), "speculant_godot.dll")),
        (os, arch) => bail!("платформа {os}/{arch} не описана в godot/speculant.gdextension"),
    }
}
