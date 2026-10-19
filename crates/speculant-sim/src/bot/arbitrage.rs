use speculant_core::content::{CityId, GoodId, Route};
use speculant_core::economy::{Currency, Markets, Money};
use speculant_core::time::GameDate;
use speculant_core::travel::Access;
use speculant_core::{Command, Game};

use super::deals;

/// Сколько сделок, лучших по марже первой штуки за день пути, бот просчитывает целиком.
const TRIPS_TO_PRICE: usize = 5;
/// Сколько дней бот не пересчитывает поездки, если везти нечего и ехать некуда.
const IDLE_DAYS: u32 = 7;

/// Перекупщик между городами: продаёт всё, с чем приехал, и везёт товар туда, где он даст больше
/// всего за день пути, ставя долю наличных и оставляя на проезд. Если из города везти нечего,
/// едет порожняком туда, откуда выгодная поездка окупит и этот переезд; если и ехать некуда —
/// неделю ждёт, пока рынки сдвинутся.
pub(crate) struct Arbitrage {
    /// Доля наличных, которую бот ставит на одну поездку.
    pub stake: f64,
    /// Самый долгий маршрут, на который бот согласен.
    pub longest_route_days: u32,
    /// До какого дня бот ждёт, не пересчитывая поездок.
    pub idle_until: Option<GameDate>,
}

/// Поездка с товаром, просчитанная по ценникам обеих толкучек сегодня.
struct Trip {
    destination: CityId,
    good: GoodId,
    quantity: u32,
    /// Прибыль за вычетом проезда в копейках дня.
    profit: i64,
    days: u32,
}

impl Trip {
    fn profit_per_day(&self) -> f64 {
        self.profit as f64 / f64::from(self.days)
    }
}

impl Arbitrage {
    pub fn trade(&mut self, game: &mut Game) {
        sell_everything(game);
        if self.idle_until.is_some_and(|until| game.today() < until) {
            return;
        }
        let cash = game.player().money().cash();
        if let Some(trip) = self.best_trip(game, game.city(), cash) {
            deals::buy(game, &trip.good, trip.quantity);
            travel(game, &trip.destination);
        } else if let Some(destination) = self.best_move(game, cash) {
            travel(game, &destination);
        } else {
            self.idle_until = Some(game.today().after_days(IDLE_DAYS));
        }
    }

    /// Маршруты из города, на которые бот согласен и которые открыты сегодня.
    fn routes_from<'a>(&self, game: &'a Game, city: &'a CityId) -> impl Iterator<Item = &'a Route> {
        let content = game.content();
        content.routes.iter().filter(move |route| {
            &route.from == city
                && route.days <= self.longest_route_days
                && Access::of(route, content, game.today()) == Access::Open
        })
    }

    /// Лучшая поездка из города `from` с наличными `cash`: сначала грубо по марже первой штуки за
    /// день пути, затем лучшие — целиком по ценникам всех штук за вычетом проезда.
    fn best_trip(&self, game: &Game, from: &CityId, cash: Money) -> Option<Trip> {
        let content = game.content();
        let today = game.today();
        let markets = game.markets();
        let on_shelf: Vec<(&GoodId, Money)> = content
            .goods
            .keys()
            .filter_map(|good| {
                let here = markets.quote(content, from, good, today)?;
                (here.stock > 0).then_some((good, here.buy))
            })
            .collect();
        let mut leads: Vec<(f64, &Route, &GoodId)> = self
            .routes_from(game, from)
            .flat_map(|route| {
                on_shelf.iter().filter_map(move |(good, buy)| {
                    let there = markets.quote(content, &route.to, good, today)?;
                    let margin = there.sell.minor_units() - buy.minor_units();
                    (margin > 0).then(|| (margin as f64 / f64::from(route.days), route, *good))
                })
            })
            .collect();
        leads.sort_by(|left, right| right.0.total_cmp(&left.0));
        leads
            .into_iter()
            .take(TRIPS_TO_PRICE)
            .filter_map(|(_, route, good)| self.price_trip(game, route, good, cash))
            .max_by(|left, right| left.profit_per_day().total_cmp(&right.profit_per_day()))
    }

    /// Поездка с этим товаром по этому маршруту: штук столько, сколько укладывается в ставку и
    /// продаётся дороже покупки; `None`, если после проезда прибыли нет.
    fn price_trip(&self, game: &Game, route: &Route, good: &GoodId, cash: Money) -> Option<Trip> {
        let content = game.content();
        let today = game.today();
        let markets = game.markets();
        let fare = content.money.from_real(route.fare, Currency::Ruble, today);
        if !cash.covers(fare) {
            return None;
        }
        let stake = (cash.minor_units() as f64 * self.stake) as i64;
        let budget = Money::new(Currency::Ruble, stake.min(cash.minus(fare).minor_units()));
        let stock = markets.city(&route.from).stock(good);
        let buys = markets.buy_tags(content, &route.from, good, stock, today)?;
        let affordable = deals::units_within(&buys, budget);
        if affordable == 0 {
            return None;
        }
        let sells = markets.sell_tags(content, &route.to, good, affordable, today)?;
        let margins: Vec<i64> = buys
            .iter()
            .zip(&sells)
            .map(|(buy, sell)| sell.minor_units() - buy.minor_units())
            .take_while(|margin| *margin > 0)
            .collect();
        let profit = margins.iter().sum::<i64>() - fare.minor_units();
        (profit > 0).then(|| Trip {
            destination: route.to.clone(),
            good: good.clone(),
            quantity: u32::try_from(margins.len()).expect("штук не больше запаса"),
            profit,
            days: route.days,
        })
    }

    /// Город, куда стоит переехать порожняком: выгодная поездка оттуда окупает переезд лучше
    /// всего за день пути обоих.
    fn best_move(&self, game: &Game, cash: Money) -> Option<CityId> {
        let content = game.content();
        let today = game.today();
        self.routes_from(game, game.city())
            .filter_map(|route| {
                let fare = content.money.from_real(route.fare, Currency::Ruble, today);
                if !cash.covers(fare) {
                    return None;
                }
                let trip = self.best_trip(game, &route.to, cash.minus(fare))?;
                let profit = trip.profit - fare.minor_units();
                let per_day = profit as f64 / f64::from(route.days + trip.days);
                (profit > 0).then(|| (per_day, route.to.clone()))
            })
            .max_by(|left, right| left.0.total_cmp(&right.0))
            .map(|(_, destination)| destination)
    }
}

fn travel(game: &mut Game, destination: &CityId) {
    game.apply(Command::Travel {
        destination: destination.to_string(),
    })
    .expect("маршрут открыт, а проезд оставлен в наличных");
}

/// Продаёт всё, что в кармане и уже продаётся: с чем приехал и что принесли события.
fn sell_everything(game: &mut Game) {
    let held: Vec<(GoodId, u32)> = game
        .player()
        .pocket()
        .goods()
        .filter(|(good, _)| Markets::is_on_sale(game.content(), good, game.today()))
        .map(|(good, quantity)| (good.clone(), quantity))
        .collect();
    for (good, quantity) in held {
        deals::sell(game, &good, quantity);
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use speculant_core::content::GoodId;
    use speculant_core::economy::Money;

    use super::Arbitrage;
    use crate::test_worlds::{new_life, real_data};

    fn trader() -> Arbitrage {
        Arbitrage {
            stake: 0.9,
            longest_route_days: u32::MAX,
            idle_until: None,
        }
    }

    #[test]
    fn the_trader_carries_only_units_that_sell_dearer_at_the_destination_than_here() {
        let mut game = new_life(&real_data(), "engineer");
        let city = game.city().clone();
        let today = game.today();
        let shelves: BTreeMap<GoodId, Vec<Money>> = game
            .content()
            .goods
            .keys()
            .filter_map(|good| {
                let stock = game.markets().city(&city).stock(good);
                let tags = game
                    .markets()
                    .buy_tags(game.content(), &city, good, stock, today)?;
                Some((good.clone(), tags))
            })
            .collect();
        let cash = game.player().money().cash();
        trader().trade(&mut game);
        let journey = game.journey().expect("выгодная поездка из Москвы есть");
        let held: Vec<_> = game.player().pocket().goods().collect();
        let [(good, quantity)] = held[..] else {
            panic!("везёт один товар: {held:?}");
        };
        let sells = game
            .markets()
            .sell_tags(game.content(), &journey.to, good, quantity, today)
            .expect("товар продаётся");
        let units = quantity as usize;
        let last = units - 1;
        assert!(
            sells[last].minor_units() > shelves[good][last].minor_units(),
            "и последняя штука продаётся дороже, чем куплена"
        );
        let earned: i64 = sells.iter().map(|tag| tag.minor_units()).sum();
        let spent = cash.minor_units() - game.player().money().cash().minor_units();
        assert!(
            earned > spent,
            "выручка {earned} больше покупки и проезда {spent}"
        );
    }

    #[test]
    fn a_resting_trader_does_not_go_anywhere_until_his_week_is_over() {
        let mut game = new_life(&real_data(), "student");
        let mut resting = Arbitrage {
            idle_until: Some(game.today().after_days(1)),
            ..trader()
        };
        resting.trade(&mut game);
        assert_eq!(game.journey(), None);
    }
}
