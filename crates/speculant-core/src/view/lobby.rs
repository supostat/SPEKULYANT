use serde::{Deserialize, Serialize};

use super::{BiographyView, NotebookEntryView, RecordsView};

/// Снимок лобби — каталог игрока открыт, жизнь не начата: главное меню и экран выбора биографии
/// видят биографии с открытиями прошлых жизней, записную книжку и рекорды.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LobbyView {
    pub biographies: Vec<BiographyView>,
    pub notebook: Vec<NotebookEntryView>,
    pub records: RecordsView,
}
