use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::project_root::project_root;
use crate::tools_manifest;

/// Скачивает пин Godot из `tools.toml`, проверяет sha256 ДО установки прав и возвращает путь к бинарнику.
/// Повторный вызов на распакованном пине ничего не качает.
pub fn fetch() -> Result<PathBuf> {
    let manifest = tools_manifest::load()?;
    let target = tools_manifest::host_target()?;
    let archive = manifest
        .godot
        .archives
        .get(target)
        .with_context(|| format!("tools.toml: нет архива Godot для цели {target}"))?;
    let install_dir = project_root()
        .join(".tools")
        .join("godot")
        .join(&manifest.godot.version);
    let binary = install_dir.join(&archive.binary);
    if binary.is_file() {
        return Ok(binary);
    }
    fs::create_dir_all(&install_dir)?;
    let archive_path = install_dir.join(&archive.file);
    if !archive_path.is_file() {
        let url = format!("{}/{}", manifest.godot.release_base_url, archive.file);
        download(&url, &archive_path)?;
    }
    verify_sha256(&archive_path, &archive.sha256)?;
    extract(&archive_path, &install_dir)?;
    if !binary.is_file() {
        bail!("после распаковки нет бинарника {}", binary.display());
    }
    make_executable(&binary)?;
    Ok(binary)
}

fn download(url: &str, destination: &Path) -> Result<()> {
    println!("Скачиваю {url}");
    let partial = destination.with_extension("part");
    let mut response = ureq::get(url)
        .call()
        .with_context(|| format!("не скачался {url}"))?;
    let mut reader = response.body_mut().as_reader();
    let mut file = File::create(&partial)?;
    io::copy(&mut reader, &mut file)?;
    fs::rename(&partial, destination)?;
    Ok(())
}

fn verify_sha256(archive_path: &Path, expected: &str) -> Result<()> {
    let mut file = File::open(archive_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 20];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    let actual: String = hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    if actual != expected {
        fs::remove_file(archive_path)?;
        bail!(
            "sha256 архива {} не совпал с tools.toml: ожидалось {expected}, получено {actual}; файл удалён",
            archive_path.display()
        );
    }
    Ok(())
}

fn extract(archive_path: &Path, install_dir: &Path) -> Result<()> {
    let file = File::open(archive_path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    archive
        .extract(install_dir)
        .with_context(|| format!("не распаковался {}", archive_path.display()))?;
    Ok(())
}

#[cfg(unix)]
fn make_executable(binary: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(binary, fs::Permissions::from_mode(0o755))?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_binary: &Path) -> Result<()> {
    Ok(())
}
