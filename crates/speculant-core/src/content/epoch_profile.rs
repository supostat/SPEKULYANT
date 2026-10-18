use serde::Deserialize;

use super::FactionId;

/// Описание эпохи из `data/epochs/<эпоха>.toml`; границы эпох — в календаре `time.toml`.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpochProfile {
    pub summary: String,
    pub threat: Threat,
    /// Раз в столько дней каждая репутация сдвигается на уровень к нулю (§9).
    pub reputation_drift_days: u32,
}

/// Главная угроза эпохи (§3): фракция, чью неприязнь показывает «Карман», и подпись индикатора.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Threat {
    pub faction: FactionId,
    pub label: String,
}
