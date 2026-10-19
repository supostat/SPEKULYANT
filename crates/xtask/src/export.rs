use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;
use zip::write::SimpleFileOptions;

use crate::build_godot;
use crate::godot_check::{import_project, run_godot};
use crate::godot_pin;
use crate::project_root::project_root;

/// Куда ложатся экспорт и архивы релиза.
pub const DIST: &str = ".tools/dist";
const GAME_FILE: &str = "Speculant";
const ARCHIVE_PREFIX: &str = "Speculant";

/// Платформа релиза (§2, §14.4): Windows x86_64, Linux x86_64 и macOS universal — у каждой свой
/// пресет в `godot/export_presets.cfg`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    WindowsX86_64,
    LinuxX86_64,
    MacosUniversal,
}

impl Platform {
    pub const ALL: [Self; 3] = [Self::WindowsX86_64, Self::LinuxX86_64, Self::MacosUniversal];

    pub fn from_name(name: &str) -> Result<Self> {
        Self::ALL
            .into_iter()
            .find(|platform| platform.name() == name)
            .with_context(|| {
                let names: Vec<&str> = Self::ALL.iter().map(|platform| platform.name()).collect();
                format!("платформы {name} нет, есть: {}", names.join(", "))
            })
    }

    /// Платформа, которую собирает эта машина: мост собирается только на родной системе.
    pub fn host() -> Result<Self> {
        match (std::env::consts::OS, std::env::consts::ARCH) {
            ("windows", "x86_64") => Ok(Self::WindowsX86_64),
            ("linux", "x86_64") => Ok(Self::LinuxX86_64),
            ("macos", _) => Ok(Self::MacosUniversal),
            (os, arch) => bail!("на {os}/{arch} игра не экспортируется: нет пресета"),
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::WindowsX86_64 => "windows-x86_64",
            Self::LinuxX86_64 => "linux-x86_64",
            Self::MacosUniversal => "macos-universal",
        }
    }

    /// Имя пресета в `godot/export_presets.cfg`.
    pub fn preset(self) -> &'static str {
        match self {
            Self::WindowsX86_64 => "Windows x86_64",
            Self::LinuxX86_64 => "Linux x86_64",
            Self::MacosUniversal => "macOS universal",
        }
    }

    /// Платформа экспорта Godot, как она записана в пресете.
    pub fn godot_platform(self) -> &'static str {
        match self {
            Self::WindowsX86_64 => "Windows Desktop",
            Self::LinuxX86_64 => "Linux",
            Self::MacosUniversal => "macOS",
        }
    }

    pub fn architecture(self) -> &'static str {
        match self {
            Self::WindowsX86_64 | Self::LinuxX86_64 => "x86_64",
            Self::MacosUniversal => "universal",
        }
    }

    /// Что выдаёт экспорт: исполняемый файл с вшитым pck или бандл macOS.
    fn export_file(self) -> String {
        match self {
            Self::WindowsX86_64 => format!("{GAME_FILE}.exe"),
            Self::LinuxX86_64 => format!("{GAME_FILE}.x86_64"),
            Self::MacosUniversal => format!("{GAME_FILE}.app"),
        }
    }

    /// Исполняемый файл экспорта. В бандле macOS он единственный в `Contents/MacOS` и назван
    /// по имени проекта.
    pub fn executable(self, export_dir: &Path) -> Result<PathBuf> {
        let exported = export_dir.join(self.export_file());
        if self != Self::MacosUniversal {
            return Ok(exported);
        }
        let binaries = exported.join("Contents").join("MacOS");
        let found: Vec<PathBuf> = fs::read_dir(&binaries)
            .with_context(|| format!("в бандле нет {}", binaries.display()))?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<_, _>>()?;
        match found.as_slice() {
            [binary] => Ok(binary.clone()),
            _ => bail!(
                "в {} не один исполняемый файл: {found:?}",
                binaries.display()
            ),
        }
    }
}

/// `cargo xtask export <платформа>`: экспорт в `.tools/dist/<платформа>/` и архив релиза
/// `.tools/dist/Speculant-<версия>-<платформа>.zip`.
pub fn run(name: &str) -> Result<()> {
    let platform = Platform::from_name(name)?;
    let export_dir = export(platform)?;
    let archive = package(platform, &export_dir)?;
    println!("export {}: {}", platform.name(), archive.display());
    Ok(())
}

/// Экспорт платформы пином Godot с его шаблонами и релизным мостом в `.tools/dist/<платформа>/`
/// — только на родной машине платформы. Прежний экспорт этой платформы заменяется.
pub fn export(platform: Platform) -> Result<PathBuf> {
    let host = Platform::host()?;
    if host != platform {
        bail!(
            "{} экспортируется на своей системе, а эта машина собирает {}",
            platform.name(),
            host.name()
        );
    }
    let binary = godot_pin::fetch()?;
    godot_pin::fetch_templates()?;
    build_godot::build_release(platform == Platform::MacosUniversal)?;
    import_project(&binary)?;
    let export_dir = project_root().join(DIST).join(platform.name());
    if export_dir.exists() {
        fs::remove_dir_all(&export_dir)?;
    }
    fs::create_dir_all(&export_dir)?;
    let output = export_dir.join(platform.export_file());
    let output = output.to_string_lossy();
    run_godot(
        &binary,
        &[
            "--headless",
            "--path",
            "godot",
            "--export-release",
            platform.preset(),
            &output,
        ],
    )?;
    let executable = platform.executable(&export_dir)?;
    if !executable.is_file() {
        bail!(
            "экспорт {} не выдал {}",
            platform.name(),
            executable.display()
        );
    }
    Ok(export_dir)
}

/// Архив релиза: всё, что выдал экспорт, с правами на исполнение.
fn package(platform: Platform, export_dir: &Path) -> Result<PathBuf> {
    let archive = project_root().join(DIST).join(format!(
        "{ARCHIVE_PREFIX}-{}-{}.zip",
        env!("CARGO_PKG_VERSION"),
        platform.name()
    ));
    let mut writer = zip::ZipWriter::new(File::create(&archive)?);
    for entry in WalkDir::new(export_dir).min_depth(1).sort_by_file_name() {
        let entry = entry?;
        let name = entry
            .path()
            .strip_prefix(export_dir)?
            .components()
            .map(|component| component.as_os_str().to_string_lossy())
            .collect::<Vec<_>>()
            .join("/");
        let options = SimpleFileOptions::default().unix_permissions(permissions(entry.path())?);
        if entry.file_type().is_dir() {
            writer.add_directory(name, options)?;
        } else {
            writer.start_file(name, options)?;
            io::copy(
                &mut File::open(entry.path())
                    .with_context(|| format!("не читается {}", entry.path().display()))?,
                &mut writer,
            )?;
        }
    }
    writer.finish()?;
    Ok(archive)
}

#[cfg(unix)]
fn permissions(path: &Path) -> Result<u32> {
    use std::os::unix::fs::PermissionsExt;
    Ok(fs::metadata(path)?.permissions().mode())
}

/// На Windows прав unix нет: исполняемость задаёт расширение.
#[cfg(not(unix))]
fn permissions(path: &Path) -> Result<u32> {
    const FILE_MODE: u32 = 0o644;
    const DIRECTORY_MODE: u32 = 0o755;
    Ok(if path.is_dir() {
        DIRECTORY_MODE
    } else {
        FILE_MODE
    })
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::Platform;

    #[test]
    fn every_platform_is_found_by_its_name() {
        for platform in Platform::ALL {
            assert_eq!(
                Platform::from_name(platform.name()).expect("платформа по имени"),
                platform
            );
        }
        assert!(Platform::from_name("macos-arm64").is_err());
    }

    #[test]
    fn a_desktop_executable_is_the_exported_file() {
        assert_eq!(
            Platform::LinuxX86_64
                .executable(Path::new("dist"))
                .expect("файл экспорта"),
            Path::new("dist/Speculant.x86_64")
        );
        assert_eq!(
            Platform::WindowsX86_64
                .executable(Path::new("dist"))
                .expect("файл экспорта"),
            Path::new("dist/Speculant.exe")
        );
    }
}
