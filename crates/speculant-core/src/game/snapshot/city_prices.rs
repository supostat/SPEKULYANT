//! Цены товара по городам в снимке (§4, §13.3 п. 1): что игрок знает о толкучках, где бывал.

use super::super::Game;
use crate::content::{City, CityId, GoodId};
use crate::economy::{Money, MoneyRules, Real, SeenPrice};
use crate::time::GameDate;
use crate::view::CityPriceView;

/// Самую низкую и самую высокую цену отмечают, только когда есть что сравнить.
const COMPARED_CITIES: usize = 2;

/// Что игрок знает о товаре в городе: день, когда видел толкучку, и ценники, если товар тогда
/// продавали.
struct Sighting {
    date: GameDate,
    price: Option<SeenPrice>,
}

impl Sighting {
    /// Ценник в рублях 1984 года — так сравниваются цены разных дней.
    fn real(&self, rules: &MoneyRules, tag: fn(&SeenPrice) -> Money) -> Option<Real> {
        self.price
            .as_ref()
            .map(|price| rules.real_value(tag(price), self.date))
    }

    /// Ценник в номинале своего дня; пусто — товара тогда не продавали.
    fn shown(&self, rules: &MoneyRules, tag: fn(&SeenPrice) -> Money) -> String {
        self.price
            .as_ref()
            .map(|price| rules.format_number(tag(price), self.date))
            .unwrap_or_default()
    }
}

/// Город в таблице цен: здесь ли игрок и что он знает о товаре там.
struct Row<'a> {
    city: &'a City,
    here: bool,
    sighting: Option<Sighting>,
}

/// Цены товара в городах, открытых сегодня: здесь — первой строкой, остальные — по названию.
/// Самая низкая цена покупки и самая высокая цена продажи сравниваются в рублях 1984 года.
pub(super) fn city_prices(game: &Game, good: &GoodId, today: GameDate) -> Vec<CityPriceView> {
    let content = &game.content;
    let rules = &content.money;
    let here = &game.world.city;
    let mut rows: Vec<Row<'_>> = content
        .cities
        .iter()
        .filter(|(_, city)| city.is_open_on(today))
        .map(|(id, city)| Row {
            city,
            here: id == here,
            sighting: sighting(game, id, good, today),
        })
        .collect();
    rows.sort_by(|left, right| {
        right
            .here
            .cmp(&left.here)
            .then_with(|| left.city.name.cmp(&right.city.name))
    });
    let real = |row: &Row<'_>, tag: fn(&SeenPrice) -> Money| {
        row.sighting
            .as_ref()
            .and_then(|sighting| sighting.real(rules, tag))
    };
    let buys: Vec<Option<Real>> = rows
        .iter()
        .map(|row| real(row, |price| price.buy))
        .collect();
    let sells: Vec<Option<Real>> = rows
        .iter()
        .map(|row| real(row, |price| price.sell))
        .collect();
    let compared = buys.iter().flatten().count() >= COMPARED_CITIES;
    let cheapest = buys.iter().flatten().min().copied().filter(|_| compared);
    let dearest = sells.iter().flatten().max().copied().filter(|_| compared);
    rows.into_iter()
        .zip(buys.into_iter().zip(sells))
        .map(|(row, (buy, sell))| {
            let shown = |tag: fn(&SeenPrice) -> Money| {
                row.sighting
                    .as_ref()
                    .map(|sighting| sighting.shown(rules, tag))
                    .unwrap_or_default()
            };
            CityPriceView {
                name: row.city.name.clone(),
                here: row.here,
                seen: row
                    .sighting
                    .as_ref()
                    .map(|sighting| sighting.date.numeric()),
                buy: shown(|price| price.buy),
                sell: shown(|price| price.sell),
                cheapest: cheapest.is_some() && buy == cheapest,
                dearest: dearest.is_some() && sell == dearest,
            }
        })
        .collect()
}

/// Здесь игрок видит толкучку сегодня, в городе, откуда уезжал, — какой она была в день отъезда;
/// где он не бывал, он не знает ничего.
fn sighting(game: &Game, city: &CityId, good: &GoodId, today: GameDate) -> Option<Sighting> {
    if *city == game.world.city {
        let quote = game.world.markets.quote(&game.content, city, good, today);
        return Some(Sighting {
            date: today,
            price: quote.map(|quote| SeenPrice {
                buy: quote.buy,
                sell: quote.sell,
            }),
        });
    }
    let market = game.world.seen_prices.market(city)?;
    Some(Sighting {
        date: market.date,
        price: market.goods.get(good).copied(),
    })
}
