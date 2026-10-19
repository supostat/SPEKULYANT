use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use sha2::{Digest, Sha256};

use crate::project_root::project_root;
use crate::tools_manifest::{self, ToolsManifest};

/// Маркер self-contained режима рядом с бинарником (на macOS — рядом с `Godot.app`, внутри бандла
/// он сломал бы подпись): данные редактора, в том числе шаблоны экспорта, живут в `editor_data/`
/// пина, а не в каталоге пользователя.
const SELF_CONTAINED_MARKER: &str = "_sc_";
const EDITOR_DATA: &str = "editor_data";
const EXPORT_TEMPLATES: &str = "export_templates";
/// Каталог шаблонов внутри архива `.tpz` и файл с версией шаблонов в нём.
const ARCHIVE_TEMPLATES_DIR: &str = "templates/";
const TEMPLATES_VERSION_FILE: &str = "version.txt";

/// Скачивает пин Godot из `tools.toml`, проверяет sha256 ДО установки прав, включает
/// self-contained режим и возвращает путь к бинарнику. Повторный вызов на распакованном пине
/// ничего не качает.
pub fn fetch() -> Result<PathBuf> {
    let manifest = tools_manifest::load()?;
    let target = tools_manifest::host_target()?;
    let archive = manifest
        .godot
        .archives
        .get(target)
        .with_context(|| format!("tools.toml: нет архива Godot для цели {target}"))?;
    let install_dir = install_dir(&manifest);
    let binary = install_dir.join(&archive.binary);
    if !binary.is_file() {
        let archive_path = download_verified(&manifest, &archive.file, &archive.sha256)?;
        extract(&archive_path, &install_dir)?;
        if !binary.is_file() {
            bail!("после распаковки нет бинарника {}", binary.display());
        }
        make_executable(&binary)?;
    }
    let marker = install_dir.join(SELF_CONTAINED_MARKER);
    if !marker.is_file() {
        File::create(&marker)?;
    }
    Ok(binary)
}

/// Ставит шаблоны экспорта пина — только настольные, `templates` из `tools.toml`, — в
/// `editor_data/export_templates/<версия шаблонов>/` и возвращает этот каталог. Архив сверяется по
/// sha256 до распаковки; повторный вызов на поставленных шаблонах ничего не качает.
pub fn fetch_templates() -> Result<PathBuf> {
    fetch()?;
    let manifest = tools_manifest::load()?;
    let templates = &manifest.godot.templates;
    let root = install_dir(&manifest)
        .join(EDITOR_DATA)
        .join(EXPORT_TEMPLATES);
    let installed = templates_dir(&root, &manifest.godot.version);
    if templates
        .files
        .iter()
        .all(|file| installed.join(file).is_file())
    {
        return Ok(installed);
    }
    let archive_path = download_verified(&manifest, &templates.file, &templates.sha256)?;
    let mut archive = zip::ZipArchive::new(File::open(&archive_path)?)?;
    let version = read_archive_text(&mut archive, TEMPLATES_VERSION_FILE)?;
    let destination = root.join(version.trim());
    if destination != installed {
        bail!(
            "шаблоны в {} — версии {}, а пин Godot — {}",
            archive_path.display(),
            version.trim(),
            manifest.godot.version
        );
    }
    fs::create_dir_all(&destination)?;
    for file in templates
        .files
        .iter()
        .chain([&TEMPLATES_VERSION_FILE.to_owned()])
    {
        let mut entry = archive
            .by_name(&format!("{ARCHIVE_TEMPLATES_DIR}{file}"))
            .with_context(|| format!("в {} нет шаблона {file}", archive_path.display()))?;
        let path = destination.join(file);
        io::copy(&mut entry, &mut File::create(&path)?)?;
        make_executable(&path)?;
    }
    Ok(destination)
}

fn install_dir(manifest: &ToolsManifest) -> PathBuf {
    project_root()
        .join(".tools")
        .join("godot")
        .join(&manifest.godot.version)
}

/// «4.7.2-stable» → `<root>/4.7.2.stable`: так Godot называет каталог шаблонов своей версии.
fn templates_dir(root: &Path, version: &str) -> PathBuf {
    root.join(version.replace('-', "."))
}

/// Архив пина в каталоге установки: скачивается, если его нет, и сверяется по sha256.
fn download_verified(manifest: &ToolsManifest, file: &str, sha256: &str) -> Result<PathBuf> {
    let install_dir = install_dir(manifest);
    fs::create_dir_all(&install_dir)?;
    let archive_path = install_dir.join(file);
    if !archive_path.is_file() {
        let url = format!("{}/{}", manifest.godot.release_base_url, file);
        download(&url, &archive_path)?;
    }
    verify_sha256(&archive_path, sha256)?;
    Ok(archive_path)
}

fn read_archive_text(archive: &mut zip::ZipArchive<File>, file: &str) -> Result<String> {
    let mut entry = archive
        .by_name(&format!("{ARCHIVE_TEMPLATES_DIR}{file}"))
        .with_context(|| format!("в архиве шаблонов нет {file}"))?;
    let mut text = String::new();
    entry.read_to_string(&mut text)?;
    Ok(text)
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::templates_dir;

    #[test]
    fn templates_live_under_the_dotted_version_of_the_pin() {
        assert_eq!(
            templates_dir(Path::new("root"), "4.7.2-stable"),
            Path::new("root").join("4.7.2.stable")
        );
    }
}
