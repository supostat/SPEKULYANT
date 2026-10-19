use serde::{Deserialize, Serialize};

/// Как соперник зарабатывает (§8).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Скупает излишек на толкучке своего города и продаёт в дефицит.
    Trader,
    /// Возит товар между своим городом и соседним.
    Shuttle,
    /// Собирает дань.
    Racketeer,
    /// Берёт на лапу.
    Official,
    /// Пускает деньги в рост.
    Banker,
}

impl Strategy {
    pub fn trades(self) -> bool {
        matches!(self, Self::Trader | Self::Shuttle)
    }

    /// Кто он — для экрана связей и соперников.
    pub fn title(self) -> &'static str {
        match self {
            Self::Trader => "торговец",
            Self::Shuttle => "челнок",
            Self::Racketeer => "рэкетир",
            Self::Official => "чиновник",
            Self::Banker => "банкир",
        }
    }
}
