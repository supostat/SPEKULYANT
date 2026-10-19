use std::collections::BTreeMap;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::Rng;

use super::{Fate, Strategy, StrategyRules};
use crate::business::{MonthlyBudget, Operation, Policies};
use crate::content::{BusinessKindId, CityId, Content, GoodId, NpcId};
use crate::economy::{Currency, Markets, Money, Real, Trader};
use crate::events::EventEngine;
use crate::reputation::Contact;
use crate::time::GameDate;

const MONTHS_IN_YEAR: f64 = 12.0;
const PERCENT: f64 = 100.0;

/// Соперник (§8): человек из шаблона со стратегией, деньгами и карманом, своим городом, для
/// челнока — вторым городом, точками своих дел в своём городе, судьбой и памятью об игроке.
/// Торгует через тот же `Trader` и те же рынки, что игрок, и ведёт дела по тем же расчётам.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rival {
    npc: NpcId,
    strategy: Strategy,
    trader: Trader,
    home: CityId,
    city: CityId,
    partner_city: Option<CityId>,
    journey: Option<(CityId, GameDate)>,
    businesses: BTreeMap<BusinessKindId, u32>,
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
            businesses: BTreeMap::new(),
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

    /// Точки дел в своём городе по типам.
    pub fn businesses(&self) -> &BTreeMap<BusinessKindId, u32> {
        &self.businesses
    }

    pub fn business_points(&self, kind: &BusinessKindId) -> u32 {
        self.businesses.get(kind).copied().unwrap_or_default()
    }

    /// Что он помнит об игроке: отношение, обида, долг, история.
    pub fn memory(&self) -> &Contact {
        &self.memory
    }

    pub fn memory_mut(&mut self) -> &mut Contact {
        &mut self.memory
    }

    /// Капитал в рублях 1984 года: деньги, товар по средней цене его города и точки дел по цене
    /// открытия.
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
        let businesses: i64 = self
            .businesses
            .iter()
            .map(|(kind, points)| {
                content.business_kinds[kind].point_cost.kopecks() * i64::from(*points)
            })
            .sum();
        Real::from_kopecks(money.kopecks() + goods.round() as i64 + businesses)
    }

    /// День соперника: выход из зоны, доход, проценты, судьба, торговля, первого числа — месяц
    /// дел, и проверка на разорение.
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
        if today.day_of_month() == 1 {
            self.run_businesses(content, markets, today);
            if rules.business_chance > 0.0 && EventEngine::roll(rng, rules.business_chance) {
                self.open_business(content, rng, today);
            }
        }
        if self.capital(content, markets, today) < content.rival_rules.ruin_below {
            self.fate = Fate::Ruined;
        }
    }

    /// Месяц дел по расчёту игрока, только грубее: политики по умолчанию, управляющий средней
    /// хватки, без происшествий и конкуренции. Убыток, который не покрыть наличными, закрывает
    /// дело.
    fn run_businesses(&mut self, content: &Content, markets: &Markets, today: GameDate) {
        let owned: Vec<(BusinessKindId, u32)> = self
            .businesses
            .iter()
            .map(|(kind, points)| (kind.clone(), *points))
            .collect();
        for (kind_id, points) in owned {
            let kind = &content.business_kinds[&kind_id];
            let policies = Policies::defaults(kind, content);
            let operation = Operation {
                points,
                revenue_percent: policies
                    .chosen(content)
                    .map(|(_, option)| option.revenue)
                    .sum(),
                skill: content.business.neutral_trait,
                wage_percent: policies.wage_percent(content),
                competitors: 0,
            };
            let budget = MonthlyBudget::plan(content, markets, kind, &self.home, operation, today);
            let (_, form) = kind
                .form_on(&content.business_forms, today)
                .expect("у открытого дела всегда есть форма собственности");
            let roof = f64::from(policies.roof_percent(content)) / PERCENT;
            let profit = budget.income * (1.0 - roof - form.tax) - budget.rent - budget.wages;
            let amount = content.money.from_real(
                Real::from_kopecks(profit.abs().round() as i64),
                Currency::Ruble,
                today,
            );
            if profit >= 0.0 {
                self.trader.receive_cash(amount);
            } else if self.trader.money().cash().covers(amount) {
                self.trader.pay_cash(amount);
            } else {
                self.businesses.remove(&kind_id);
            }
        }
    }

    /// Точка дела, на которое хватает денег с запасом правил, — тип выбирает случай из тех, что
    /// сейчас можно открыть и где точек ещё не предел.
    fn open_business(&mut self, content: &Content, rng: &mut ChaCha8Rng, today: GameDate) {
        let reserve = 1.0 + content.business.rival_reserve;
        let cash = self.trader.money().cash();
        let affordable: Vec<&BusinessKindId> = content
            .business_kinds
            .iter()
            .filter(|(id, kind)| {
                let with_reserve =
                    Real::from_kopecks((kind.point_cost.kopecks() as f64 * reserve).round() as i64);
                kind.form_on(&content.business_forms, today).is_some()
                    && self.business_points(id) < kind.max_points
                    && cash.covers(
                        content
                            .money
                            .from_real(with_reserve, Currency::Ruble, today),
                    )
            })
            .map(|(id, _)| id)
            .collect();
        if affordable.is_empty() {
            return;
        }
        let kind = affordable[(rng.next_u64() % affordable.len() as u64) as usize].clone();
        let cost = content.money.from_real(
            content.business_kinds[&kind].point_cost,
            Currency::Ruble,
            today,
        );
        self.trader.pay_cash(cost);
        *self.businesses.entry(kind).or_default() += 1;
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
