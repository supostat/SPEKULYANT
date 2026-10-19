use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::Rng;

use super::{Fate, Strategy, StrategyRules};
use crate::content::{CityId, Content, GoodId, NpcId};
use crate::economy::{Currency, Markets, Money, Real, Trader};
use crate::events::EventEngine;
use crate::reputation::Contact;
use crate::time::GameDate;

const MONTHS_IN_YEAR: f64 = 12.0;
const PERCENT: f64 = 100.0;

/// Соперник (§8): человек из шаблона со стратегией, деньгами и карманом, своим городом, для
/// челнока — вторым городом, судьбой и памятью об игроке. Торгует через тот же `Trader` и те же
/// рынки, что игрок.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rival {
    npc: NpcId,
    strategy: Strategy,
    trader: Trader,
    home: CityId,
    city: CityId,
    partner_city: Option<CityId>,
    journey: Option<(CityId, GameDate)>,
    fate: Fate,
    memory: Contact,
}

impl Rival {
    pub fn new(
        npc: NpcId,
        strategy: Strategy,
        cash: Money,
        home: CityId,
        partner_city: Option<CityId>,
        content: &Content,
    ) -> Self {
        Self {
            memory: Contact::new(npc.clone()),
            npc,
            strategy,
            trader: Trader::new(cash, content),
            city: home.clone(),
            home,
            partner_city,
            journey: None,
            fate: Fate::Active,
        }
    }

    pub fn npc(&self) -> &NpcId {
        &self.npc
    }

    pub fn strategy(&self) -> Strategy {
        self.strategy
    }

    pub fn trader(&self) -> &Trader {
        &self.trader
    }

    /// Где он сейчас; в пути — город отправления.
    pub fn city(&self) -> &CityId {
        &self.city
    }

    pub fn home(&self) -> &CityId {
        &self.home
    }

    pub fn partner_city(&self) -> Option<&CityId> {
        self.partner_city.as_ref()
    }

    pub fn is_on_the_road(&self) -> bool {
        self.journey.is_some()
    }

    pub fn fate(&self) -> Fate {
        self.fate
    }

    /// Что он помнит об игроке: отношение, обида, долг, история.
    pub fn memory(&self) -> &Contact {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Contact {
        &mut self.memory
    }

    /// Капитал в рублях 1984 года: деньги и товар по средней цене его города.
    pub fn capital(&self, content: &Content, markets: &Markets, today: GameDate) -> Real {
        let money = self.trader.money().real_total(&content.money, today);
        let goods: f64 = self
            .trader
            .pocket()
            .goods()
            .map(|(good, quantity)| {
                let price = markets
                    .normal_kopecks(content, &self.city, good, today)
                    .expect("товар в кармане куплен, значит уже в продаже");
                price * f64::from(quantity)
            })
            .sum();
        Real::from_kopecks(money.kopecks() + goods.round() as i64)
    }

    /// День соперника: выход из зоны, доход, проценты, судьба, торговля и проверка на разорение.
    /// Все розыгрыши идут подряд из одного потока, поэтому день одного соперника не зависит от
    /// того, торговал ли другой.
    pub fn live_day(
        &mut self,
        content: &Content,
        markets: &mut Markets,
        rng: &mut ChaCha8Rng,
        today: GameDate,
    ) {
        if let Fate::Prison { until } = self.fate
            && today >= until
        {
            self.fate = Fate::Active;
        }
        if !self.fate.is_active() {
            return;
        }
        let rules = content.rival_rules.strategies[&self.strategy];
        if rules.daily_income.is_positive() {
            let income = content
                .money
                .from_real(rules.daily_income, Currency::Ruble, today);
            self.trader.receive_cash(income);
        }
        if today.day_of_month() == 1 && rules.annual_percent > 0.0 {
            let cash = self.trader.money().cash();
            let interest =
                cash.minor_units() as f64 * rules.annual_percent / PERCENT / MONTHS_IN_YEAR;
            self.trader
                .receive_cash(Money::new(Currency::Ruble, interest.round() as i64));
        }
        if self.meets_fate(&rules, rng, content, today) {
            return;
        }
        if self.strategy.trades() && EventEngine::roll(rng, rules.trade_chance) {
            match &self.partner_city {
                Some(_) => self.shuttle(content, markets, rng, rules.lot, today),
                None => self.trade_at_home(content, markets, rng, rules.lot, today),
            }
        }
        if self.capital(content, markets, today) < content.rival_rules.ruin_below {
            self.fate = Fate::Ruined;
        }
    }

    /// Гибель, зона и депутатство — каждый день со своими вероятностями; `true`, если судьба
    /// пришла.
    fn meets_fate(
        &mut self,
        rules: &StrategyRules,
        rng: &mut ChaCha8Rng,
        content: &Content,
        today: GameDate,
    ) -> bool {
        let dies = EventEngine::roll(rng, rules.death_chance);
        let jailed = EventEngine::roll(rng, rules.prison_chance);
        let elected = EventEngine::roll(rng, rules.deputy_chance);
        self.fate = if dies {
            Fate::Dead
        } else if jailed {
            Fate::Prison {
                until: today.after_days(content.rival_rules.prison_days),
            }
        } else if elected {
            Fate::Deputy
        } else {
            return false;
        };
        true
    }

    /// Торговля в своём городе: случайный товар; излишек на рынке он скупает — сколько по карману,
    /// в дефицит — продаёт.
    fn trade_at_home(
        &mut self,
        content: &Content,
        markets: &mut Markets,
        rng: &mut ChaCha8Rng,
        lot: u32,
        today: GameDate,
    ) {
        let on_sale: Vec<&GoodId> = content
            .goods
            .keys()
            .filter(|good| Markets::is_on_sale(content, good, today))
            .collect();
        if on_sale.is_empty() {
            return;
        }
        let good = on_sale[(rng.next_u64() % on_sale.len() as u64) as usize];
        let normal = content.goods[good].stock;
        let stock = markets.city(&self.city).stock(good);
        let held = self.trader.pocket().quantity(good);
        if held > 0 && stock < normal {
            let quantity = held.min(lot).min(normal - stock);
            self.trader
                .sell(markets, content, &self.city, good, quantity, today)
                .ok();
        } else if stock > normal {
            let quantity = self.affordable(
                content,
                markets,
                good,
                lot.min(stock - normal),
                Money::zero(Currency::Ruble),
                today,
            );
            if quantity > 0 {
                self.trader
                    .buy(markets, content, &self.city, good, quantity, today)
                    .ok();
            }
        }
    }

    /// Сколько штук из `wanted` по нынешним ценникам по карману, если отложить `reserve`: каждая
    /// следующая дороже.
    fn affordable(
        &self,
        content: &Content,
        markets: &Markets,
        good: &GoodId,
        wanted: u32,
        reserve: Money,
        today: GameDate,
    ) -> u32 {
        let wanted = wanted.min(markets.city(&self.city).stock(good));
        let Some(tags) = markets.buy_tags(content, &self.city, good, wanted, today) else {
            return 0;
        };
        let cash = self.trader.money().cash();
        if !cash.covers(reserve) {
            return 0;
        }
        let mut left = cash.minus(reserve);
        let mut count = 0;
        for tag in tags {
            if !left.covers(tag) {
                break;
            }
            left = left.minus(tag);
            count += 1;
        }
        count
    }

    /// Челнок: дома скупает товар, который во втором городе дороже всего относительно дома, и
    /// едет туда; там продаёт всё и возвращается. В пути ничего не делает.
    fn shuttle(
        &mut self,
        content: &Content,
        markets: &mut Markets,
        rng: &mut ChaCha8Rng,
        lot: u32,
        today: GameDate,
    ) {
        if let Some((destination, arrives)) = &self.journey {
            if today < *arrives {
                return;
            }
            self.city = destination.clone();
            self.journey = None;
        }
        let partner = self
            .partner_city
            .clone()
            .expect("челнок с маршрутом знает второй город");
        if self.city == self.home {
            let Some(fare) = self.fare(content, &partner, today) else {
                self.trade_at_home(content, markets, rng, lot, today);
                return;
            };
            if let Some(good) = self.best_cargo(content, markets, &partner, today) {
                let quantity = self.affordable(content, markets, &good, lot, fare, today);
                if quantity > 0 {
                    self.trader
                        .buy(markets, content, &self.city, &good, quantity, today)
                        .ok();
                }
            }
            if self.trader.pocket().goods().next().is_some() {
                self.depart(content, partner, today);
            }
        } else {
            let held: Vec<(GoodId, u32)> = self
                .trader
                .pocket()
                .goods()
                .map(|(good, quantity)| (good.clone(), quantity))
                .collect();
            for (good, quantity) in held {
                self.trader
                    .sell(markets, content, &self.city, &good, quantity, today)
                    .ok();
            }
            let home = self.home.clone();
            if !self.depart(content, home, today) {
                self.trade_at_home(content, markets, rng, lot, today);
            }
        }
    }

    /// Товар, который во втором городе дороже, чем дома, сильнее всех; `None`, если такого нет.
    fn best_cargo(
        &self,
        content: &Content,
        markets: &Markets,
        partner: &CityId,
        today: GameDate,
    ) -> Option<GoodId> {
        content
            .goods
            .keys()
            .filter_map(|good| {
                let here = markets.normal_kopecks(content, &self.city, good, today)?;
                let there = markets.normal_kopecks(content, partner, good, today)?;
                (there > here).then_some((good, there / here))
            })
            .max_by(|left, right| left.1.total_cmp(&right.1))
            .map(|(good, _)| good.clone())
    }

    /// Проезд отсюда в город по маршруту; `None`, если маршрута нет.
    fn fare(&self, content: &Content, destination: &CityId, today: GameDate) -> Option<Money> {
        content
            .routes
            .iter()
            .find(|route| route.from == self.city && &route.to == destination)
            .map(|route| content.money.from_real(route.fare, Currency::Ruble, today))
    }

    /// Выезд по маршруту с оплатой проезда; `true`, если уехал.
    fn depart(&mut self, content: &Content, destination: CityId, today: GameDate) -> bool {
        let Some(route) = content
            .routes
            .iter()
            .find(|route| route.from == self.city && route.to == destination)
        else {
            return false;
        };
        let fare = content.money.from_real(route.fare, Currency::Ruble, today);
        if !self.trader.money().cash().covers(fare) {
            return false;
        }
        self.trader.pay_cash(fare);
        self.journey = Some((destination, today.after_days(route.days)));
        true
    }
}
