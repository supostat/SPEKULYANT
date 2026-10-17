use std::collections::BTreeMap;

use serde::Deserialize;

use crate::time::GameDate;

/// Город (§4): множители цен по категориям товаров и доступность по датам.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct City {
    pub name: String,
    pub region: String,
    #[serde(default)]
    pub multipliers: BTreeMap<String, f64>,
    /// До этой даты город закрыт (Польша — с 1990–91, Китай — с 1992).
    pub opens: Option<GameDate>,
    /// До этой даты в город попадают только по блату (ГДР и Венгрия до 1990).
    pub connections_only_until: Option<GameDate>,
}
