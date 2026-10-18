use super::{GameDate, TimeRulesError};

/// Эпоха — каркас правил мира (§3): с какого дня она начинается.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Epoch {
    pub id: String,
    pub title: String,
    pub starts: GameDate,
}

/// Эпохи по порядку: первая начинается в первый день игры, каждая следующая — позже предыдущей.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EpochTable {
    epochs: Vec<Epoch>,
}

impl EpochTable {
    pub fn new(epochs: Vec<Epoch>) -> Result<Self, TimeRulesError> {
        let first = epochs.first().ok_or(TimeRulesError::NoEpochs)?;
        if first.starts != GameDate::START {
            return Err(TimeRulesError::FirstEpochStart {
                id: first.id.clone(),
            });
        }
        for (index, pair) in epochs.windows(2).enumerate() {
            if pair[1].starts <= pair[0].starts {
                return Err(TimeRulesError::EpochOrder {
                    id: pair[1].id.clone(),
                });
            }
            if epochs[..=index]
                .iter()
                .any(|earlier| earlier.id == pair[1].id)
            {
                return Err(TimeRulesError::DuplicateEpoch {
                    id: pair[1].id.clone(),
                });
            }
        }
        Ok(Self { epochs })
    }

    pub fn epochs(&self) -> &[Epoch] {
        &self.epochs
    }

    pub fn epoch(&self, id: &str) -> Option<&Epoch> {
        self.epochs.iter().find(|epoch| epoch.id == id)
    }

    /// Эпоха, в которой лежит день игры.
    pub fn epoch_on(&self, date: GameDate) -> &Epoch {
        self.epochs
            .iter()
            .rev()
            .find(|epoch| epoch.starts <= date)
            .expect("день игры не раньше первого дня первой эпохи")
    }
}
