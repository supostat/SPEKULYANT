use serde::Deserialize;

/// Описание эпохи из `data/epochs/<эпоха>.toml`; границы эпох — в календаре `time.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpochProfile {
    pub summary: String,
}
