use serde::Deserialize;

use super::CategoryId;
use crate::economy::PriceCurve;

/// Товар (§5.3): категория для множителей городов, обычный запас на толкучке и кривая цены по
/// эпохам в рублях 1984 года.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Good {
    pub name: String,
    /// Короткое имя для окна «Карман»: «Джинсы» вместо «Джинсы (фирма)».
    pub short_name: String,
    pub category: CategoryId,
    pub unit: String,
    /// Сколько штук обычно на толкучке города; запас меньше — цена выше, больше — ниже.
    pub stock: u32,
    pub curve: PriceCurve,
}
