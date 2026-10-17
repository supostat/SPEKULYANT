use std::collections::BTreeMap;

use crate::content::GoodId;

/// Товар при себе: окно «Карман» (§13.3 п. 1).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Pocket {
    goods: BTreeMap<GoodId, u32>,
}

impl Pocket {
    pub fn quantity(&self, good: &GoodId) -> u32 {
        self.goods.get(good).copied().unwrap_or(0)
    }

    /// Товар при себе по порядку идентификаторов, без закончившихся.
    pub fn goods(&self) -> impl Iterator<Item = (&GoodId, u32)> {
        self.goods.iter().map(|(good, quantity)| (good, *quantity))
    }

    pub fn add(&mut self, good: &GoodId, quantity: u32) {
        let held = self.goods.entry(good.clone()).or_insert(0);
        *held = held.saturating_add(quantity);
    }

    /// Убирает товар; отдают не больше, чем есть — это проверяет сделка.
    pub fn remove(&mut self, good: &GoodId, quantity: u32) {
        let left = self
            .quantity(good)
            .checked_sub(quantity)
            .expect("из кармана отдают не больше, чем в нём есть");
        if left == 0 {
            self.goods.remove(good);
        } else {
            self.goods.insert(good.clone(), left);
        }
    }
}
