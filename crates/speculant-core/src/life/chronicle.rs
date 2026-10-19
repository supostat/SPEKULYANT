use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{DealRecord, GoodPrices, PriceNote};
use crate::content::{BusinessKindId, CityId, GoodId};
use crate::economy::Real;
use crate::time::GameDate;

/// Летопись жизни для итогов (§11.2) и записной книжки (§11.3): во что обошёлся товар в кармане,
/// лучшая и худшая продажи, первое своё дело и почём штуки покупались и продавались.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Chronicle {
    /// Сколько стоил весь товар каждого вида в кармане, в копейках 1984 года.
    cost_basis: BTreeMap<GoodId, Real>,
    best_deal: Option<DealRecord>,
    worst_deal: Option<DealRecord>,
    first_business: Option<(GameDate, BusinessKindId)>,
    prices: BTreeMap<GoodId, GoodPrices>,
}

impl Chronicle {
    pub fn best_deal(&self) -> Option<&DealRecord> {
        self.best_deal.as_ref()
    }

    pub fn worst_deal(&self) -> Option<&DealRecord> {
        self.worst_deal.as_ref()
    }

    /// Когда и какое дело игрок открыл первым.
    pub fn first_business(&self) -> Option<&(GameDate, BusinessKindId)> {
        self.first_business.as_ref()
    }

    /// Почём штуки товаров покупались и продавались в этой жизни.
    pub fn prices(&self) -> &BTreeMap<GoodId, GoodPrices> {
        &self.prices
    }

    /// Куплено `quantity` штук на толкучке `city` за `cost`.
    pub fn bought(
        &mut self,
        good: &GoodId,
        quantity: u32,
        cost: Real,
        date: GameDate,
        city: &CityId,
    ) {
        let basis = self.cost_basis.entry(good.clone()).or_insert(Real::ZERO);
        *basis = Real::from_kopecks(basis.kopecks() + cost.kopecks());
        self.prices
            .entry(good.clone())
            .or_default()
            .bought(unit_price(cost, quantity, date, city));
    }

    /// Продано `quantity` из `held` на толкучке `city` за `proceeds`: прибыль — выручка минус
    /// средняя цена покупки проданного; товар, полученный даром, продаётся в чистую прибыль.
    pub fn sold(
        &mut self,
        good: &GoodId,
        quantity: u32,
        held: u32,
        proceeds: Real,
        date: GameDate,
        city: &CityId,
    ) {
        self.prices
            .entry(good.clone())
            .or_default()
            .sold(unit_price(proceeds, quantity, date, city));
        let cost = self.take_basis(good, quantity, held);
        let deal = DealRecord {
            good: good.clone(),
            date,
            profit: Real::from_kopecks(proceeds.kopecks() - cost.kopecks()),
        };
        if self
            .best_deal
            .as_ref()
            .is_none_or(|best| deal.profit > best.profit)
        {
            self.best_deal = Some(deal.clone());
        }
        if self
            .worst_deal
            .as_ref()
            .is_none_or(|worst| deal.profit < worst.profit)
        {
            self.worst_deal = Some(deal);
        }
    }

    /// Товар ушёл из кармана не продажей — отдан в событии: его цена покупки уходит вместе с ним.
    pub fn gave_away(&mut self, good: &GoodId, quantity: u32, held: u32) {
        self.take_basis(good, quantity, held);
    }

    pub fn opened_business(&mut self, kind: &BusinessKindId, date: GameDate) {
        self.first_business
            .get_or_insert_with(|| (date, kind.clone()));
    }

    /// Цена покупки `quantity` штук из `held` — средняя, она уходит из учёта; отдают не больше,
    /// чем держат, и держат хотя бы одну.
    fn take_basis(&mut self, good: &GoodId, quantity: u32, held: u32) -> Real {
        let Some(basis) = self.cost_basis.get_mut(good) else {
            return Real::ZERO;
        };
        let share = basis.kopecks() * i64::from(quantity) / i64::from(held);
        *basis = Real::from_kopecks(basis.kopecks() - share);
        if quantity >= held {
            self.cost_basis.remove(good);
        }
        Real::from_kopecks(share)
    }
}

/// Цена одной штуки сделки.
fn unit_price(total: Real, quantity: u32, date: GameDate, city: &CityId) -> PriceNote {
    PriceNote {
        price: Real::from_kopecks(total.kopecks() / i64::from(quantity)),
        date,
        city: city.clone(),
    }
}
