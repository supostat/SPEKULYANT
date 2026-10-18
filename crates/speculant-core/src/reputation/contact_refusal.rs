use std::fmt;

use crate::time::GameDate;

/// Почему человек не помогает.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ContactRefusal {
    Lost,
    Away { until: GameDate },
    Offended,
    OwedTooMuch,
}

impl fmt::Display for ContactRefusal {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Lost => write!(formatter, "этого человека больше нет рядом"),
            Self::Away { until } => write!(formatter, "не на связи до {}", until.numeric()),
            Self::Offended => write!(formatter, "обижен и помогать не станет"),
            Self::OwedTooMuch => write!(formatter, "за тобой и так слишком много услуг"),
        }
    }
}

impl std::error::Error for ContactRefusal {}
