use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{CityMarket, Money, PriceDrift, Shock};
use crate::content::{CityId, Content, GoodId};
use crate::rng::WorldRng;
use crate::time::GameDate;

const MARKET_STREAM: &str = "market";

/// Ценники следующей штуки на толкучке и запас товара.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Quote {
    pub buy: Money,
    pub sell: Money,
    pub stock: u32,
}

/// Рынки всех городов, действующие шоки цен и дрейф (§5.2). Цена в городе — кривая товара,
/// множитель города по категории, дрейф и сезон, шоки событий и местный запас; соперники торгуют
/// по тем же правилам.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Markets {
    cities: BTreeMap<CityId, CityMarket>,
    shocks: Vec<Shock>,
    drift: PriceDrift,
}

impl Markets {
    /// Рынки каждого города с обычным запасом каждого товара; цены дрейфуют по сиду мира.
    pub fn new(content: &Content, seed: u64) -> Self {
        Self {
            cities: content
                .cities
                .keys()
                .map(|city| (city.clone(), CityMarket::new(&content.goods)))
                .collect(),
            shocks: Vec::new(),
            drift: PriceDrift::new(seed),
        }
    }

    /// Действующие и отжившие, но ещё не убранные шоки — от старых к новым.
    pub fn shocks(&self) -> &[Shock] {
        &self.shocks
    }

    pub fn city(&self, city: &CityId) -> &CityMarket {
        self.cities
            .get(city)
            .expect("рынок есть у каждого города контента")
    }

    /// Товар уже появился: его кривая цены началась.
    pub fn is_on_sale(content: &Content, good: &GoodId, date: GameDate) -> bool {
        content.goods[good].curve.kopecks_on(date).is_some()
    }

    /// Средняя реальная цена штуки при обычном запасе в копейках 1984 года: кривая товара,
    /// множитель города по категории, дрейф и сезон и действующие шоки; `None`, пока товара ещё
    /// нет.
    pub fn normal_kopecks(
        &self,
        content: &Content,
        city: &CityId,
        good: &GoodId,
        date: GameDate,
    ) -> Option<f64> {
        let item = &content.goods[good];
        let curve = item.curve.kopecks_on(date)?;
        let multiplier = content.cities[city].multiplier(&item.category);
        let drift = self.drift.factor(
            &content.drift,
            good,
            &content.categories[&item.category],
            date,
        );
        let shocks: f64 = self
            .shocks
            .iter()
            .filter(|shock| shock.applies_to(good, city))
            .map(|shock| shock.factor)
            .product();
        Some(curve * multiplier * drift * shocks)
    }

    /// Ценники `quantity` штук подряд при покупке; `None`, пока товара ещё нет. Хватает ли запаса,
    /// проверяет покупатель.
    pub fn buy_tags(
        &self,
        content: &Content,
        city: &CityId,
        good: &GoodId,
        quantity: u32,
        date: GameDate,
    ) -> Option<Vec<Money>> {
        let normal = self.normal_kopecks(content, city, good, date)?;
        let prices = self.city(city).buy_prices(
            good,
            normal,
            content.goods[good].stock,
            &content.market,
            quantity,
        );
        Some(tags(content, prices, date))
    }

    /// Реальная цена следующей штуки при покупке в копейках 1984 года — то, что история цен
    /// записывает раз в неделю; `None`, пока товара ещё нет.
    pub fn buy_real_kopecks(
        &self,
        content: &Content,
        city: &CityId,
        good: &GoodId,
        date: GameDate,
    ) -> Option<f64> {
        let normal = self.normal_kopecks(content, city, good, date)?;
        Some(self.city(city).buy_price_after(
            good,
            normal,
            content.goods[good].stock,
            &content.market,
            0,
        ))
    }

    /// Ценники `quantity` штук подряд при продаже; `None`, пока товара ещё нет.
    pub fn sell_tags(
        &self,
        content: &Content,
        city: &CityId,
        good: &GoodId,
        quantity: u32,
        date: GameDate,
    ) -> Option<Vec<Money>> {
        let normal = self.normal_kopecks(content, city, good, date)?;
        let prices = self.city(city).sell_prices(
            good,
            normal,
            content.goods[good].stock,
            &content.market,
            quantity,
        );
        Some(tags(content, prices, date))
    }

    /// Строка толкучки: ценники следующей штуки и запас; `None`, пока товара ещё нет.
    pub fn quote(
        &self,
        content: &Content,
        city: &CityId,
        good: &GoodId,
        date: GameDate,
    ) -> Option<Quote> {
        let buy = self.buy_tags(content, city, good, 1, date)?;
        let sell = self.sell_tags(content, city, good, 1, date)?;
        Some(Quote {
            buy: buy[0],
            sell: sell[0],
            stock: self.city(city).stock(good),
        })
    }

    /// Товар куплен с рынка города.
    pub fn take(&mut self, city: &CityId, good: &GoodId, quantity: u32) {
        self.city_mut(city).take(good, quantity);
    }

    /// Товар продан на рынок города — им же пользуются соперники, вычерпывая и заваливая рынок.
    pub fn put(&mut self, city: &CityId, good: &GoodId, quantity: u32) {
        self.city_mut(city).put(good, quantity);
    }

    /// Шок цены: товар дорожает или дешевеет в `factor` раз — в городе или везде (`city = None`),
    /// на `days` дней или навсегда (`days = None`).
    pub fn apply_shock(
        &mut self,
        good: GoodId,
        city: Option<CityId>,
        factor: f64,
        days: Option<u32>,
        today: GameDate,
    ) {
        self.shocks.push(Shock {
            good,
            city,
            factor,
            until: days.map(|days| today.after_days(days)),
        });
    }

    /// Новый день: кончаются сроки шоков, запасы всех городов возвращаются к обычным и колеблются
    /// по сиду мира.
    pub fn next_day(&mut self, content: &Content, rng: &mut WorldRng, date: GameDate) {
        self.shocks.retain(|shock| !shock.is_over_on(date));
        let stream = rng.stream(MARKET_STREAM);
        for market in self.cities.values_mut() {
            market.next_day(&content.goods, &content.market, stream);
        }
    }

    fn city_mut(&mut self, city: &CityId) -> &mut CityMarket {
        self.cities
            .get_mut(city)
            .expect("рынок есть у каждого города контента")
    }
}

fn tags(content: &Content, prices: Vec<f64>, date: GameDate) -> Vec<Money> {
    prices
        .into_iter()
        .map(|price| content.money.price_tag(price, date))
        .collect()
}
