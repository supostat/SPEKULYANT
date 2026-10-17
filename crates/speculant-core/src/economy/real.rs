use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

const KOPECKS_PER_RUBLE: i64 = 100;

/// Сумма в реальных единицах — копейках 1984 года. Номинал на ценнике, деноминация и валюты —
/// слой поверх (§5.1). В данных пишется в рублях 1984 года: `price = 160` или `price = 4.5`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Real(i64);

impl Real {
    pub const ZERO: Self = Self(0);

    pub fn from_kopecks(kopecks: i64) -> Self {
        Self(kopecks)
    }

    pub fn kopecks(self) -> i64 {
        self.0
    }

    /// Рубли 1984 года с округлением до копейки; `None` для бесконечности, NaN и переполнения.
    pub fn from_rubles(rubles: f64) -> Option<Self> {
        let kopecks = (rubles * KOPECKS_PER_RUBLE as f64).round();
        (kopecks.is_finite() && kopecks.abs() < i64::MAX as f64).then_some(Self(kopecks as i64))
    }

    pub fn is_positive(self) -> bool {
        self.0 > 0
    }

    pub fn is_negative(self) -> bool {
        self.0 < 0
    }
}

impl Serialize for Real {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_f64(self.0 as f64 / KOPECKS_PER_RUBLE as f64)
    }
}

impl<'de> Deserialize<'de> for Real {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(RublesVisitor)
    }
}

struct RublesVisitor;

impl Visitor<'_> for RublesVisitor {
    type Value = Real;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "сумму в рублях 1984 года числом")
    }

    fn visit_i64<E: de::Error>(self, rubles: i64) -> Result<Real, E> {
        rubles
            .checked_mul(KOPECKS_PER_RUBLE)
            .map(Real)
            .ok_or_else(|| E::custom(format!("{rubles} р. не помещается в счёт ядра")))
    }

    fn visit_u64<E: de::Error>(self, rubles: u64) -> Result<Real, E> {
        let rubles = i64::try_from(rubles)
            .map_err(|_| E::custom(format!("{rubles} р. не помещается в счёт ядра")))?;
        self.visit_i64(rubles)
    }

    fn visit_f64<E: de::Error>(self, rubles: f64) -> Result<Real, E> {
        Real::from_rubles(rubles).ok_or_else(|| E::custom(format!("{rubles} — не сумма в рублях")))
    }
}
