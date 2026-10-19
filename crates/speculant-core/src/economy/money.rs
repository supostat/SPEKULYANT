use serde::{Deserialize, Serialize};

use super::Currency;

/// Сумма в валюте: рубли — в копейках до деноминации 1998 года, чеки — в копейках чека, доллары —
/// в центах. Складывать и сравнивать можно только суммы одной валюты; иное — ошибка в коде ядра.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    currency: Currency,
    minor_units: i64,
}

impl Money {
    pub fn new(currency: Currency, minor_units: i64) -> Self {
        Self {
            currency,
            minor_units,
        }
    }

    pub fn zero(currency: Currency) -> Self {
        Self::new(currency, 0)
    }

    pub fn currency(self) -> Currency {
        self.currency
    }

    pub fn minor_units(self) -> i64 {
        self.minor_units
    }

    pub fn is_zero(self) -> bool {
        self.minor_units == 0
    }

    pub fn plus(self, other: Self) -> Self {
        self.assert_same_currency(other);
        Self::new(
            self.currency,
            self.minor_units.saturating_add(other.minor_units),
        )
    }

    pub fn minus(self, other: Self) -> Self {
        self.assert_same_currency(other);
        Self::new(
            self.currency,
            self.minor_units.saturating_sub(other.minor_units),
        )
    }

    /// Хватает ли этой суммы, чтобы отдать `other`.
    pub fn covers(self, other: Self) -> bool {
        self.assert_same_currency(other);
        self.minor_units >= other.minor_units
    }

    fn assert_same_currency(self, other: Self) {
        assert_eq!(
            self.currency, other.currency,
            "суммы разных валют не складываются и не сравниваются"
        );
    }
}
