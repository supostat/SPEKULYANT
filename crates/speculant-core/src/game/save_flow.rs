//! Сохранения (§11.4, решение 15.10): свободный режим сохраняется в именованные слоты, ironman —
//! сам в свой единственный слот после каждого дня и поступка и без отката; метапрогрессия лежит
//! отдельным файлом и пишется, когда жизнь кончилась.

use std::sync::Arc;

use super::Game;
use super::world::World;
use crate::command::LifeSettings;
use crate::command_error::CommandError;
use crate::content::Content;
use crate::meta::Meta;
use crate::save::{LIFE, META, SaveError, Storage};

/// Единственный слот ironman-жизни; свободному режиму он не достаётся.
const IRONMAN_SLOT: &str = "ironman";
const AUTOSAVE_FAILED: &str = "Автосохранение не удалось";
const META_NOT_SAVED: &str = "Знания прошлых жизней не сохранились";

impl Game {
    /// Новая жизнь с каталогом игрока: метапрогрессия — с диска, ironman-жизнь сразу ложится в
    /// свой слот.
    pub fn open(
        settings: &LifeSettings,
        content: Arc<Content>,
        storage: Storage,
    ) -> Result<Self, CommandError> {
        let meta = read_meta(&storage)?;
        Self::start_life(settings, content, meta, Some(storage))
    }

    /// Жизнь из слота с метапрогрессией с диска — так игра продолжается после перерыва.
    pub fn resume(
        content: Arc<Content>,
        storage: Storage,
        slot: &str,
    ) -> Result<Self, CommandError> {
        let meta = read_meta(&storage)?;
        let world = read_world(&storage, slot, &content)?;
        Ok(Self {
            content,
            meta,
            storage: Some(storage),
            world,
        })
    }

    /// Сохранение свободного режима в слот; ironman сохраняется сам, а кончившуюся жизнь
    /// сохранять незачем.
    pub(super) fn save(&mut self, slot: &str) -> Result<(), CommandError> {
        let storage = self.storage.as_ref().ok_or(CommandError::NoStorage)?;
        if self.world.ironman {
            return Err(CommandError::IronmanSavesItself);
        }
        if self.world.summary.is_some() {
            return Err(CommandError::LifeOver);
        }
        if slot == IRONMAN_SLOT {
            return Err(CommandError::InvalidSlot {
                slot: slot.to_owned(),
            });
        }
        write_world(storage, slot, &self.world, &self.content).map_err(|error| match error {
            SaveError::InvalidSlot(slot) => CommandError::InvalidSlot { slot },
            other => CommandError::SaveFailed {
                reason: other.to_string(),
            },
        })
    }

    /// Жизнь из слота вместо нынешней; идущую ironman-жизнь не откатить.
    pub(super) fn load(&mut self, slot: &str) -> Result<(), CommandError> {
        let storage = self.storage.as_ref().ok_or(CommandError::NoStorage)?;
        if self.world.ironman && self.world.summary.is_none() {
            return Err(CommandError::IronmanNoRollback);
        }
        self.world = read_world(storage, slot, &self.content)?;
        Ok(())
    }

    /// Ironman-жизнь, пока идёт, ложится в свой слот; если диск подвёл, об этом пишет лента.
    pub(super) fn autosave(&mut self) {
        let Some(storage) = &self.storage else {
            return;
        };
        if !self.world.ironman || self.world.summary.is_some() {
            return;
        }
        if let Err(error) = write_world(storage, IRONMAN_SLOT, &self.world, &self.content) {
            self.log(format!("{AUTOSAVE_FAILED}: {error}"));
        }
    }

    /// Жизнь кончилась: метапрогрессия ложится на диск, ironman-жизнь стирается из своего слота.
    pub(super) fn store_after_life(&mut self) {
        let Some(storage) = &self.storage else {
            return;
        };
        let mut result = storage.write_meta(&META.encode(None, &self.meta));
        if self.world.ironman {
            result = result.and(storage.delete_life(IRONMAN_SLOT));
        }
        if let Err(error) = result {
            self.log(format!("{META_NOT_SAVED}: {error}"));
        }
    }
}

/// Метапрогрессия из каталога игрока; файла нет — прошлых жизней не было.
pub(super) fn read_meta(storage: &Storage) -> Result<Meta, CommandError> {
    let unreadable = |error: SaveError| CommandError::MetaUnreadable {
        reason: error.to_string(),
    };
    match storage.read_meta().map_err(unreadable)? {
        None => Ok(Meta::default()),
        Some(text) => META.decode(&text, None).map_err(unreadable),
    }
}

fn read_world(storage: &Storage, slot: &str, content: &Content) -> Result<World, CommandError> {
    let unreadable = |error: SaveError| match error {
        SaveError::Missing => CommandError::NoSuchSave {
            slot: slot.to_owned(),
        },
        SaveError::InvalidSlot(slot) => CommandError::InvalidSlot { slot },
        other => CommandError::SaveUnreadable {
            slot: slot.to_owned(),
            reason: other.to_string(),
        },
    };
    let text = storage.read_life(slot).map_err(unreadable)?;
    LIFE.decode(&text, Some(content.fingerprint))
        .map_err(unreadable)
}

fn write_world(
    storage: &Storage,
    slot: &str,
    world: &World,
    content: &Content,
) -> Result<(), SaveError> {
    storage.write_life(slot, &LIFE.encode(Some(content.fingerprint), world))
}
