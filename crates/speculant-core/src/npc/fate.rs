use crate::time::GameDate;

/// Судьба соперника (§8): в деле, в зоне до даты, разорился, погиб или ушёл в депутаты.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
