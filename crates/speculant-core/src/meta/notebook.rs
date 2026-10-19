use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::content::{GoodId, NpcId};
use crate::life::{DealRecord, GoodPrices};

/// Записная книжка (§11.3): товары — почём их брали и сбывали, люди — последняя запись истории
/// знакомства, схемы — лучшая сделка с каждым товаром за все жизни.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Notebook {
    goods: BTreeMap<GoodId, GoodPrices>,
    people: BTreeMap<NpcId, String>,
    schemes: BTreeMap<GoodId, DealRecord>,
}

impl Notebook {
    pub fn goods(&self) -> &BTreeMap<GoodId, GoodPrices> {
        &self.goods
    }

    pub fn people(&self) -> &BTreeMap<NpcId, String> {
        &self.people
    }

    pub fn schemes(&self) -> &BTreeMap<GoodId, DealRecord> {
        &self.schemes
    }

    /// Цены жизни ложатся поверх прошлых: остаются самая дешёвая покупка и самая дорогая
    /// продажа.
    pub fn note_prices(&mut self, prices: &BTreeMap<GoodId, GoodPrices>) {
        for (good, known) in prices {
            self.goods.entry(good.clone()).or_default().merge(known);
        }
    }

    /// О человеке остаётся то, что знала последняя жизнь.
    pub fn note_person(&mut self, npc: NpcId, note: String) {
        self.people.insert(npc, note);
    }

    /// Схема — сделка с прибылью; остаётся лучшая по товару.
    pub fn note_deal(&mut self, deal: &DealRecord) {
        if !deal.profit.is_positive() {
            return;
        }
        let best = self.schemes.get(&deal.good);
        if best.is_none_or(|best| deal.profit > best.profit) {
            self.schemes.insert(deal.good.clone(), deal.clone());
        }
    }
}
