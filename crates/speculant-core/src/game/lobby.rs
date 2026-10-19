//! Лобби (§11.3, §13.3 п. 11): каталог игрока открыт, жизнь не начата. Главное меню и экран выбора
//! биографии видят то, что оставили прошлые жизни, — открытые биографии, записную книжку и
//! рекорды; жизнь начинается отдельно, `Game::open` или `Game::resume` в том же каталоге.

use std::sync::Arc;

use super::save_flow::read_meta;
use super::snapshot;
use crate::command_error::CommandError;
use crate::content::Content;
use crate::meta::Meta;
use crate::save::Storage;
use crate::view::LobbyView;

#[derive(Debug)]
pub struct Lobby {
    content: Arc<Content>,
    meta: Meta,
}

impl Lobby {
    /// Метапрогрессия из каталога игрока; в пустом каталоге прошлых жизней ещё не было.
    pub fn open(content: Arc<Content>, storage: &Storage) -> Result<Self, CommandError> {
        let meta = read_meta(storage)?;
        Ok(Self { content, meta })
    }

    /// Снимок лобби для интерфейса.
    pub fn view(&self) -> LobbyView {
        snapshot::lobby(&self.content, &self.meta)
    }
}
