use serde::{Deserialize, Serialize};

use super::FateKind;
use crate::time::GameDate;

/// Судьба соперника (§8): в деле, в зоне до даты, разорился, погиб или ушёл в депутаты.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Fate {
    Active,
    Prison { until: GameDate },
    Ruined,
    Dead,
    Deputy,
}

impl Fate {
    pub fn is_active(self) -> bool {
        self == Self::Active
    }

    /// Судьба без срока зоны — так её называют условия событий.
    pub fn kind(self) -> FateKind {
        match self {
            Self::Active => FateKind::Active,
            Self::Prison { .. } => FateKind::Prison,
            Self::Ruined => FateKind::Ruined,
            Self::Dead => FateKind::Dead,
            Self::Deputy => FateKind::Deputy,
        }
    }

    /// Строка для экрана соперников.
    pub fn status(self) -> String {
        match self {
            Self::Active => "в деле".to_owned(),
            Self::Prison { until } => format!("в зоне до {}", until.numeric()),
            Self::Ruined => "разорился".to_owned(),
            Self::Dead => "погиб".to_owned(),
            Self::Deputy => "депутат".to_owned(),
        }
    }
}
