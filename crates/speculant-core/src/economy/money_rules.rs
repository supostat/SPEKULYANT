use serde::Deserialize;

use super::{Currency, Denomination, Money, Real, RubleCurve};
use crate::time::GameDate;

/// Сумма до 10 единиц показывается с копейками, крупнее — целыми единицами.
const WHOLE_UNITS_FROM_MINOR: i64 = 10 * Currency::MINOR_UNITS;
const DIGIT_GROUP: usize = 3;

/// Номинал и валюты (§5.1, §6) из `data/economy/money.toml`: индекс цен с деноминацией и курсы
/// чеков и доллара — всё в рублях на ценнике по годам. Здесь реальная стоимость становится суммой
/// в валюте и обратно, а сумма — строкой для игрока.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoneyRules {
    pub denomination: Denomination,
    /// Индекс цен: рублей на ценнике за рубль 1984 года.
    pub inflation: RubleCurve,
    pub checks_rate: RubleCurve,
    pub dollar_rate: RubleCurve,
}

impl MoneyRules {
    /// Индекс цен на дату: рублей до деноминации за рубль 1984 года.
    pub fn index_on(&self, date: GameDate) -> f64 {
        self.inflation.old_rubles_on(date, self.denomination)
    }

    /// Реальная стоимость суммы в этот день.
    pub fn real_value(&self, money: Money, date: GameDate) -> Real {
        let old_kopecks =
            money.minor_units() as f64 * self.old_rubles_per_unit(money.currency(), date);
        Real::from_kopecks(rounded(old_kopecks / self.index_on(date)))
    }

    /// Сумма в валюте, которая в этот день стоит `real`.
    pub fn from_real(&self, real: Real, currency: Currency, date: GameDate) -> Money {
        let old_kopecks = real.kopecks() as f64 * self.index_on(date);
        Money::new(
            currency,
            rounded(old_kopecks / self.old_rubles_per_unit(currency, date)),
        )
    }

    /// Обмен по курсу дня.
    pub fn exchange(&self, money: Money, to: Currency, date: GameDate) -> Money {
        let old_kopecks =
            money.minor_units() as f64 * self.old_rubles_per_unit(money.currency(), date);
        Money::new(
            to,
            rounded(old_kopecks / self.old_rubles_per_unit(to, date)),
        )
    }

    /// Целые единицы валюты так, как их называет игрок: «положить 500» — 500 рублей своего дня.
    /// Немыслимо большое число упирается в предел счёта и дальше отказывается как нехватка денег.
    pub fn whole_units(&self, currency: Currency, units: u64, date: GameDate) -> Money {
        let per_unit = Currency::MINOR_UNITS * self.shown_divisor(currency, date);
        let minor_units = i64::try_from(units)
            .unwrap_or(i64::MAX)
            .saturating_mul(per_unit);
        Money::new(currency, minor_units)
    }

    /// Ценник штуки товара в рублях дня по реальной цене в копейках 1984 года: до 10 р. — с
    /// копейками, дороже — целыми рублями, дешевле копейки товар не бывает.
    pub fn price_tag(&self, real_kopecks: f64, date: GameDate) -> Money {
        let divisor = self.denomination.divisor_on(date);
        let shown_kopecks = real_kopecks * self.index_on(date) / divisor as f64;
        let rounded_kopecks = if shown_kopecks >= WHOLE_UNITS_FROM_MINOR as f64 {
            rounded(shown_kopecks / Currency::MINOR_UNITS as f64) * Currency::MINOR_UNITS
        } else {
            rounded(shown_kopecks).max(1)
        };
        Money::new(Currency::Ruble, rounded_kopecks * divisor)
    }

    /// Сумма с валютой так, как её видит игрок: «1 240 р.», «4.50 р.», «$ 18 400», «85 чек.».
    pub fn format_money(&self, money: Money, date: GameDate) -> String {
        money.currency().with_sign(&self.format_number(money, date))
    }

    /// Число без валюты — для столбцов цен и строк вроде «Чеки: 85»: до 10 единиц с копейками,
    /// если они есть («4.50», «3»), крупнее — целыми с разбивкой по три цифры («1 240»). Рубли
    /// после деноминации — новые.
    pub fn format_number(&self, money: Money, date: GameDate) -> String {
        let shown = divide_rounded(
            money.minor_units(),
            self.shown_divisor(money.currency(), date),
        );
        let has_kopecks = shown % Currency::MINOR_UNITS != 0;
        if shown.abs() < WHOLE_UNITS_FROM_MINOR && has_kopecks {
            let sign = if shown < 0 { "-" } else { "" };
            let minor = shown.unsigned_abs();
            let per_unit = Currency::MINOR_UNITS.unsigned_abs();
            format!("{sign}{}.{:02}", minor / per_unit, minor % per_unit)
        } else {
            grouped(divide_rounded(shown, Currency::MINOR_UNITS))
        }
    }

    /// Во сколько раз единица на ценнике крупнее единицы счёта ядра: деноминация трогает только рубли.
    fn shown_divisor(&self, currency: Currency, date: GameDate) -> i64 {
        match currency {
            Currency::Ruble => self.denomination.divisor_on(date),
            Currency::Checks | Currency::Dollar => 1,
        }
    }

    fn old_rubles_per_unit(&self, currency: Currency, date: GameDate) -> f64 {
        match currency {
            Currency::Ruble => 1.0,
            Currency::Checks => self.checks_rate.old_rubles_on(date, self.denomination),
            Currency::Dollar => self.dollar_rate.old_rubles_on(date, self.denomination),
        }
    }
}

fn rounded(value: f64) -> i64 {
    value.round() as i64
}

/// Деление с округлением половины от нуля.
fn divide_rounded(value: i64, divisor: i64) -> i64 {
    let half = divisor / 2;
    if value >= 0 {
        value.saturating_add(half) / divisor
    } else {
        value.saturating_sub(half) / divisor
    }
}

/// Целое с разбивкой по три цифры пробелом: «1 240», «18 400 000».
fn grouped(value: i64) -> String {
    let digits = value.unsigned_abs().to_string();
    let mut text = String::with_capacity(digits.len() + digits.len() / DIGIT_GROUP + 1);
    if value < 0 {
        text.push('-');
    }
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(DIGIT_GROUP) {
            text.push(' ');
        }
        text.push(digit);
    }
    text
}
