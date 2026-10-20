use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::Rng;

use super::MarketRules;
use crate::content::{Good, GoodId};

/// Цена штуки считается по запасу посередине между «до» и «после» сделки, поэтому купить штуку и
/// тут же продать её обратно всегда дороже, чем выручить.
const HALF_UNIT: f64 = 0.5;
/// Пустой рынок продаёт вдвое дороже средней цены.
const EMPTY_MARKET_MARKUP: f64 = 2.0;

/// Толкучка одного города (§5.2 п. 3): сколько каждого товара сейчас у торговцев. Спрос выражен в
/// цене: чем больше товара против обычного запаса, тем дешевле его и покупают, и продают. Крупная
/// продажа заваливает рынок и обваливает цену; за дни запас возвращается к обычному.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CityMarket {
    stock: BTreeMap<GoodId, u32>,
}

impl CityMarket {
    /// Рынок с обычным запасом каждого товара.
    pub fn new(goods: &BTreeMap<GoodId, Good>) -> Self {
        Self {
            stock: goods
                .iter()
                .map(|(id, good)| (id.clone(), good.stock))
                .collect(),
        }
    }

    pub fn stock(&self, good: &GoodId) -> u32 {
        *self
            .stock
            .get(good)
            .expect("на рынке города есть каждый товар контента")
    }

    /// Реальные цены штук при покупке `quantity` подряд в копейках 1984 года: запас тает, каждая
    /// следующая штука дороже. `normal_kopecks` — средняя цена при обычном запасе `normal_stock`.
    pub fn buy_prices(
        &self,
        good: &GoodId,
        normal_kopecks: f64,
        normal_stock: u32,
        rules: &MarketRules,
        quantity: u32,
    ) -> Vec<f64> {
        (0..quantity)
            .map(|bought| self.buy_price_after(good, normal_kopecks, normal_stock, rules, bought))
            .collect()
    }

    /// Реальная цена штуки при покупке, когда `bought` штук уже купили подряд.
    pub fn buy_price_after(
        &self,
        good: &GoodId,
        normal_kopecks: f64,
        normal_stock: u32,
        rules: &MarketRules,
        bought: u32,
    ) -> f64 {
        let between = f64::from(self.stock(good)) - f64::from(bought) - HALF_UNIT;
        unit_price(normal_kopecks, normal_stock, between) * rules.markup()
    }

    /// Реальные цены штук при продаже `quantity` подряд: рынок заваливается, каждая следующая дешевле.
    pub fn sell_prices(
        &self,
        good: &GoodId,
        normal_kopecks: f64,
        normal_stock: u32,
        rules: &MarketRules,
        quantity: u32,
    ) -> Vec<f64> {
        let stock = f64::from(self.stock(good));
        (0..quantity)
            .map(|sold| {
                let between = stock + f64::from(sold) + HALF_UNIT;
                unit_price(normal_kopecks, normal_stock, between) * rules.markdown()
            })
            .collect()
    }

    /// Товар ушёл с рынка: его купили.
    pub fn take(&mut self, good: &GoodId, quantity: u32) {
        let stock = self.stock_mut(good);
        *stock = stock
            .checked_sub(quantity)
            .expect("с рынка забирают не больше запаса");
    }

    /// Товар пришёл на рынок: его продали.
    pub fn put(&mut self, good: &GoodId, quantity: u32) {
        let stock = self.stock_mut(good);
        *stock = stock.saturating_add(quantity);
    }

    /// День рынка: запас каждого товара возвращается к обычному на долю `recovery` отклонения, но
    /// хотя бы на штуку, и случайно колеблется в пределах `daily_noise` обычного запаса.
    pub fn next_day(
        &mut self,
        goods: &BTreeMap<GoodId, Good>,
        rules: &MarketRules,
        rng: &mut ChaCha8Rng,
    ) {
        for (id, good) in goods {
            let normal = i64::from(good.stock);
            let stock = self.stock_mut(id);
            let current = i64::from(*stock);
            let gap = normal - current;
            let step = (gap.abs() as f64 * rules.recovery).ceil() as i64;
            let recovered = current + gap.signum() * step.min(gap.abs());
            let amplitude = (normal as f64 * rules.daily_noise).round() as i64;
            let noise = if amplitude > 0 {
                random_between(rng, amplitude)
            } else {
                0
            };
            *stock = u32::try_from((recovered + noise).max(0)).unwrap_or(u32::MAX);
        }
    }

    fn stock_mut(&mut self, good: &GoodId) -> &mut u32 {
        self.stock
            .get_mut(good)
            .expect("на рынке города есть каждый товар контента")
    }
}

/// Цена штуки при запасе `stock_between`: при обычном запасе — средняя цена, на пустом рынке —
/// вдвое дороже, на заваленном — дешевле в разы.
fn unit_price(normal_kopecks: f64, normal_stock: u32, stock_between: f64) -> f64 {
    let normal_stock = f64::from(normal_stock);
    normal_kopecks * EMPTY_MARKET_MARKUP * normal_stock / (stock_between + normal_stock)
}

/// Равновероятное целое от `-amplitude` до `amplitude`.
fn random_between(rng: &mut ChaCha8Rng, amplitude: i64) -> i64 {
    let width = (2 * amplitude + 1) as u64;
    (rng.next_u64() % width) as i64 - amplitude
}
