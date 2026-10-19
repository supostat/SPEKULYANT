use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{Currency, Money, MoneyRules, Real};
use crate::content::{MoneyPlace, MoneyPlaceId};
use crate::time::GameDate;

const MONTHS_IN_YEAR: f64 = 12.0;
const PERCENT: f64 = 100.0;

/// Деньги одного человека (§6): наличные рубли при себе и вклады по местам хранения — каждый в
/// валюте своего места. Риски мест (кража, заморозка, отзыв лицензии, дефолт) приходят событиями.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MoneyStore {
    cash: Money,
    balances: BTreeMap<MoneyPlaceId, Money>,
}

impl MoneyStore {
    /// Наличные в рублях и пустые вклады во всех местах хранения.
    pub fn new(cash: Money, places: &BTreeMap<MoneyPlaceId, MoneyPlace>) -> Self {
        assert_eq!(cash.currency(), Currency::Ruble, "наличные — рубли");
        Self {
            cash,
            balances: places
                .iter()
                .map(|(id, place)| (id.clone(), Money::zero(place.currency)))
                .collect(),
        }
    }

    pub fn cash(&self) -> Money {
        self.cash
    }

    pub fn balance(&self, place: &MoneyPlaceId) -> Money {
        *self
            .balances
            .get(place)
            .expect("вклад заведён для каждого места хранения контента")
    }

    /// Всё, что лежит в местах хранения этой валюты: столько чеков или долларов у человека.
    pub fn holdings(&self, currency: Currency) -> Money {
        self.balances
            .values()
            .filter(|balance| balance.currency() == currency)
            .fold(Money::zero(currency), |total, balance| total.plus(*balance))
    }

    /// Отдаёт наличные; хватает ли их, проверяет вызывающий.
    pub fn pay(&mut self, amount: Money) {
        self.cash = self.cash.minus(amount);
    }

    pub fn receive(&mut self, amount: Money) {
        self.cash = self.cash.plus(amount);
    }

    /// Кладёт сумму в валюте места.
    pub fn put(&mut self, place: &MoneyPlaceId, amount: Money) {
        let balance = self.balance_mut(place);
        *balance = balance.plus(amount);
    }

    /// Забирает сумму в валюте места; хватает ли вклада, проверяет вызывающий.
    pub fn take(&mut self, place: &MoneyPlaceId, amount: Money) {
        let balance = self.balance_mut(place);
        *balance = balance.minus(amount);
    }

    /// Проценты за месяц по годовой ставке каждого места на эту дату, в валюте места.
    pub fn accrue_monthly_interest(
        &mut self,
        places: &BTreeMap<MoneyPlaceId, MoneyPlace>,
        date: GameDate,
    ) {
        for (id, balance) in &mut self.balances {
            let monthly_share = places[id].annual_percent_on(date) / PERCENT / MONTHS_IN_YEAR;
            let interest = (balance.minor_units() as f64 * monthly_share).round() as i64;
            *balance = balance.plus(Money::new(balance.currency(), interest));
        }
    }

    /// Реальная стоимость всех денег — наличных и вкладов по курсу дня.
    pub fn real_total(&self, rules: &MoneyRules, date: GameDate) -> Real {
        let kopecks = std::iter::once(&self.cash)
            .chain(self.balances.values())
            .map(|money| rules.real_value(*money, date).kopecks())
            .sum();
        Real::from_kopecks(kopecks)
    }

    fn balance_mut(&mut self, place: &MoneyPlaceId) -> &mut Money {
        self.balances
            .get_mut(place)
            .expect("вклад заведён для каждого места хранения контента")
    }
}
