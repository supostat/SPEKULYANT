use std::fs;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use super::SaveError;

const SAVES_DIRECTORY: &str = "saves";
const META_FILE: &str = "meta.spk";
const EXTENSION: &str = "spk";
const TEMPORARY_EXTENSION: &str = "spk.tmp";
const LONGEST_SLOT: usize = 32;

/// Каталог игрока (§11.4): жизни — `saves/<слот>.spk`, метапрогрессия — `meta.spk`. Запись идёт
/// во временный файл и переименованием на место, так что оборванная запись не портит прежний
/// файл. Имя слота проверяется здесь, на границе с файловой системой.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Storage {
    directory: PathBuf,
}

impl Storage {
    pub fn new(directory: impl Into<PathBuf>) -> Self {
        Self {
            directory: directory.into(),
        }
    }

    pub fn read_life(&self, slot: &str) -> Result<String, SaveError> {
        read(&self.life_path(slot)?)?.ok_or(SaveError::Missing)
    }

    pub fn write_life(&self, slot: &str, text: &str) -> Result<(), SaveError> {
        write_atomically(&self.life_path(slot)?, text)
    }

    /// Стирает жизнь из слота; пустой слот стирать нечего.
    pub fn delete_life(&self, slot: &str) -> Result<(), SaveError> {
        match fs::remove_file(self.life_path(slot)?) {
            Err(error) if error.kind() != ErrorKind::NotFound => {
                Err(SaveError::Io(error.to_string()))
            }
            _ => Ok(()),
        }
    }

    /// Метапрогрессия; `None`, пока ни одна жизнь не кончилась.
    pub fn read_meta(&self) -> Result<Option<String>, SaveError> {
        read(&self.directory.join(META_FILE))
    }

    pub fn write_meta(&self, text: &str) -> Result<(), SaveError> {
        write_atomically(&self.directory.join(META_FILE), text)
    }

    fn life_path(&self, slot: &str) -> Result<PathBuf, SaveError> {
        let valid = !slot.is_empty()
            && slot.len() <= LONGEST_SLOT
            && slot.chars().all(|character| {
                character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
            });
        if !valid {
            return Err(SaveError::InvalidSlot(slot.to_owned()));
        }
        Ok(self
            .directory
            .join(SAVES_DIRECTORY)
            .join(slot)
            .with_extension(EXTENSION))
    }
}

fn read(path: &Path) -> Result<Option<String>, SaveError> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(Some(text)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(SaveError::Io(error.to_string())),
    }
}

fn write_atomically(path: &Path, text: &str) -> Result<(), SaveError> {
    let io = |error: std::io::Error| SaveError::Io(error.to_string());
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(io)?;
    }
    let temporary = path.with_extension(TEMPORARY_EXTENSION);
    fs::write(&temporary, text).map_err(io)?;
    fs::rename(&temporary, path).map_err(io)
}
