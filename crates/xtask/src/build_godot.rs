use std::fs;
use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use cargo_metadata::MetadataCommand;
use cargo_metadata::camino::Utf8PathBuf;

use crate::project_root::project_root;

const BRIDGE_CRATE: &str = "speculant-godot";
const MACOS_TARGETS: [&str; 2] = ["aarch64-apple-darwin", "x86_64-apple-darwin"];

#[derive(Clone, Copy)]
pub enum Profile {
    Debug,
    Release,
}

impl Profile {
    fn directory(self) -> &'static str {
        match self {
            Self::Debug => "debug",
            Self::Release => "release",
        }
    }
}

/// `cargo xtask build-godot [--release] [--universal]`.
pub fn run(flags: &[String]) -> Result<()> {
    let mut profile = Profile::Debug;
    let mut universal = false;
    for flag in flags {
        match flag.as_str() {
            "--release" => profile = Profile::Release,
            "--universal" => universal = true,
            unknown => {
                bail!("build-godot: неизвестный флаг {unknown}, есть --release и --universal")
            }
        }
    }
    let library = if universal {
        build_macos_universal(profile)?
    } else {
        build_host(profile)?
    };
    println!("Мост: {}", library.display());
    Ok(())
}

/// Отладочная сборка моста для хоста — её загружают все безголовые гейты.
pub fn build_host_debug() -> Result<PathBuf> {
    build_host(Profile::Debug)
}

/// Релизная сборка моста для экспорта: на macOS — universal, на остальных — для хоста.
pub fn build_release(universal: bool) -> Result<PathBuf> {
    if universal {
        build_macos_universal(Profile::Release)
    } else {
        build_host(Profile::Release)
    }
}

/// Собирает cdylib моста для хоста и кладёт туда, куда смотрит `godot/speculant.gdextension`.
fn build_host(profile: Profile) -> Result<PathBuf> {
    cargo_build(profile, None)?;
    let (platform_dir, library_file) = host_library_layout()?;
    let built = target_directory()?
        .join(profile.directory())
        .join(library_file);
    let destination = library_destination(&platform_dir, library_file)?;
    fs::copy(&built, &destination).with_context(|| format!("нет собранной библиотеки {built}"))?;
    Ok(destination)
}

/// Собирает мост под arm64 и x86_64 и склеивает их `lipo` в universal-библиотеку macOS.
fn build_macos_universal(profile: Profile) -> Result<PathBuf> {
    if std::env::consts::OS != "macos" {
        bail!("universal-сборка моста делается только на macOS");
    }
    install_rust_targets(&MACOS_TARGETS)?;
    let library_file = "libspeculant_godot.dylib";
    let target_dir = target_directory()?;
    let mut slices = Vec::new();
    for target in MACOS_TARGETS {
        cargo_build(profile, Some(target))?;
        slices.push(
            target_dir
                .join(target)
                .join(profile.directory())
                .join(library_file),
        );
    }
    let destination = library_destination("macos", library_file)?;
    let status = Command::new("lipo")
        .arg("-create")
        .args(slices.iter().map(|slice| slice.as_str()))
        .arg("-output")
        .arg(&destination)
        .status()
        .context("lipo не запускается: нужны Xcode Command Line Tools")?;
    if !status.success() {
        bail!("lipo завершился с кодом {status}");
    }
    Ok(destination)
}

/// Цели кросс-сборки ставятся в закреплённый тулчейн по требованию: на Linux и Windows они не нужны.
fn install_rust_targets(targets: &[&str]) -> Result<()> {
    let status = Command::new("rustup")
        .args(["target", "add"])
        .args(targets)
        .current_dir(project_root())
        .status()
        .context("rustup не запускается: universal-сборке нужны цели обеих архитектур")?;
    if !status.success() {
        bail!(
            "rustup target add {} завершился с кодом {status}",
            targets.join(" ")
        );
    }
    Ok(())
}

fn cargo_build(profile: Profile, target: Option<&str>) -> Result<()> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Command::new(cargo);
    command
        .args(["build", "-p", BRIDGE_CRATE])
        .current_dir(project_root());
    if matches!(profile, Profile::Release) {
        command.arg("--release");
    }
    if let Some(target) = target {
        command.args(["--target", target]);
    }
    let status = command.status()?;
    if !status.success() {
        bail!("cargo build -p {BRIDGE_CRATE} завершился с кодом {status}");
    }
    Ok(())
}

fn target_directory() -> Result<Utf8PathBuf> {
    Ok(MetadataCommand::new()
        .manifest_path(project_root().join("Cargo.toml"))
        .no_deps()
        .exec()?
        .target_directory)
}

fn library_destination(platform_dir: &str, library_file: &str) -> Result<PathBuf> {
    let destination_dir = project_root().join("godot").join("bin").join(platform_dir);
    fs::create_dir_all(&destination_dir)?;
    Ok(destination_dir.join(library_file))
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
