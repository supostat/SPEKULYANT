//! Выгода торговли (§5.2): лучший рейс с кошельком `[trade] purse` раз в неделю каждой эпохи — по
//! открытым маршрутам из открытых городов, при обычном запасе обеих толкучек и дрейфе цен по сиду,
//! за вычетом проезда. Выгода — прибыль к кошельку: деньги, которые рынок не смог принять,
//! простаивают.

use serde::Serialize;
use speculant_core::Content;
use speculant_core::content::{GoodId, Route};
use speculant_core::economy::{Currency, Markets, Real};
use speculant_core::time::GameDate;
use speculant_core::travel::Access;

/// Замеры — раз в неделю.
const WEEK_DAYS: u32 = 7;

/// Лучшие рейсы по эпохам, у которых есть конец.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct TradeReport {
    pub epochs: Vec<EpochTrade>,
}

/// Эпоха глазами перекупщика: медиана выгоды лучшего рейса по неделям — его прибыль к кошельку.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct EpochTrade {
    pub epoch: String,
    pub title: String,
    pub weeks: usize,
    pub trip_return: f64,
}

impl TradeReport {
    pub fn build(content: &Content, purse: Real, seed: u64) -> Self {
        let markets = Markets::new(content, seed);
        let epochs = content
            .time
            .epochs
            .epochs()
            .windows(2)
            .map(|pair| {
                let mut returns: Vec<f64> = weeks(pair[0].starts, pair[1].starts)
                    .map(|date| {
                        let wallet = content.money.from_real(purse, Currency::Ruble, date);
                        best_profit(content, &markets, wallet.minor_units(), date) as f64
                            / wallet.minor_units() as f64
                    })
                    .collect();
                returns.sort_by(f64::total_cmp);
                EpochTrade {
                    epoch: pair[0].id.clone(),
                    title: pair[0].title.clone(),
                    weeks: returns.len(),
                    trip_return: returns[returns.len() / 2],
                }
            })
            .collect();
        Self { epochs }
    }

    pub fn epoch(&self, id: &str) -> Option<&EpochTrade> {
        self.epochs.iter().find(|epoch| epoch.epoch == id)
    }
}

/// Первые дни недель от `starts` до `ends`, не включая его.
fn weeks(starts: GameDate, ends: GameDate) -> impl Iterator<Item = GameDate> {
    (0..)
        .map(move |week| starts.after_days(week * WEEK_DAYS))
        .take_while(move |date| *date < ends)
}

/// Прибыль самого прибыльного рейса дня с кошельком `wallet` в копейках дня по открытым маршрутам —
/// из открытого города в открытый; если выгодного нет — наименее убыточного.
fn best_profit(content: &Content, markets: &Markets, wallet: i64, date: GameDate) -> i64 {
    content
        .routes
        .iter()
        .filter(|route| {
            content.cities[&route.from].is_open_on(date)
                && Access::of(route, content, date) == Access::Open
        })
        .flat_map(|route| {
            content
                .goods
                .keys()
                .filter_map(move |good| trip_profit(content, markets, route, good, wallet, date))
        })
        .max()
        .expect("в любой день открыт хотя бы один маршрут из Москвы")
}

/// Прибыль рейса с кошельком `wallet` за вычетом проезда: штуки покупаются подряд, пока каждая
/// продаётся дороже, чем куплена, и пока хватает кошелька; `None`, если товара ещё нет.
fn trip_profit(
    content: &Content,
    markets: &Markets,
    route: &Route,
    good: &GoodId,
    wallet: i64,
    date: GameDate,
) -> Option<i64> {
    let fare = content
        .money
        .from_real(route.fare, Currency::Ruble, date)
        .minor_units();
    let budget = wallet - fare;
    let stock = content.goods[good].stock;
    let buys = markets.buy_tags(content, &route.from, good, stock, date)?;
    let sells = markets.sell_tags(content, &route.to, good, stock, date)?;
    let mut cost = 0;
    let mut profit = 0;
    for (buy, sell) in buys
        .iter()
        .zip(&sells)
        .map(|(buy, sell)| (buy.minor_units(), sell.minor_units()))
    {
        if sell <= buy || cost + buy > budget {
            break;
        }
        cost += buy;
        profit += sell - buy;
    }
    Some(profit - fare)
}
