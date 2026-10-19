use std::collections::BTreeMap;

use rand_chacha::rand_core::Rng;

use super::{Rival, Strategy};
use crate::content::{BusinessKindId, CityId, Content, NpcId, RivalCondition};
use crate::economy::{Currency, Markets};
use crate::rng::WorldRng;
use crate::time::GameDate;
use crate::travel::Access;

const NPC_STREAM: &str = "npc";

/// Соперники мира (§8): состав выбирается по сиду из шаблонов со стратегией, дальше каждый живёт
/// свой день.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rivals {
    rivals: BTreeMap<NpcId, Rival>,
}

impl Rivals {
    /// Состав по сиду: от `min_rivals` до `max_rivals` человек, сколько позволит пул шаблонов; у
    /// каждого — город из открытых, у челнока — второй город на открытом маршруте.
    pub fn new(content: &Content, rng: &mut WorldRng, today: GameDate) -> Self {
        let rules = &content.rival_rules;
        let stream = rng.stream(NPC_STREAM);
        let pool: Vec<&NpcId> = content
            .npcs
            .iter()
            .filter(|(_, template)| template.strategy.is_some())
            .map(|(id, _)| id)
            .collect();
        let homes: Vec<&CityId> = content
            .cities
            .iter()
            .filter(|(_, city)| {
                city.opens.is_none_or(|opens| opens <= today)
                    && city
                        .connections_only_until
                        .is_none_or(|until| until <= today)
            })
            .map(|(id, _)| id)
            .collect();
        let spread = u64::from(rules.max_rivals - rules.min_rivals) + 1;
        let wanted = u64::from(rules.min_rivals) + stream.next_u64() % spread;
        let count = usize::try_from(wanted)
            .unwrap_or(usize::MAX)
            .min(pool.len());
        let count = if homes.is_empty() { 0 } else { count };
        let mut order: Vec<usize> = (0..pool.len()).collect();
        for slot in 0..count {
            let remaining = (pool.len() - slot) as u64;
            let pick = slot + (stream.next_u64() % remaining) as usize;
            order.swap(slot, pick);
        }
        let mut rivals = BTreeMap::new();
        for &index in &order[..count] {
            let npc = pool[index];
            let template = &content.npcs[npc];
            let strategy = template.strategy.expect("в пуле только соперники");
            let home = homes[(stream.next_u64() % homes.len() as u64) as usize].clone();
            let partner_city = (strategy == Strategy::Shuttle)
                .then(|| {
                    let open: Vec<&CityId> = content
                        .routes
                        .iter()
                        .filter(|route| {
                            route.from == home && Access::of(route, content, today) == Access::Open
                        })
                        .map(|route| &route.to)
                        .collect();
                    (!open.is_empty())
                        .then(|| open[(stream.next_u64() % open.len() as u64) as usize].clone())
                })
                .flatten();
            let capital = template
                .capital
                .expect("у соперника есть капитал — это проверяет загрузка");
            let cash = content.money.from_real(capital, Currency::Ruble, today);
            rivals.insert(
                npc.clone(),
                Rival::new(npc.clone(), strategy, cash, home, partner_city, content),
            );
        }
        Self { rivals }
    }

    pub fn get(&self, npc: &NpcId) -> Option<&Rival> {
        self.rivals.get(npc)
    }

    pub fn get_mut(&mut self, npc: &NpcId) -> Option<&mut Rival> {
        self.rivals.get_mut(npc)
    }

    pub fn iter(&self) -> impl Iterator<Item = &Rival> {
        self.rivals.values()
    }

    pub fn len(&self) -> usize {
        self.rivals.len()
    }

    pub fn is_empty(&self) -> bool {
        self.rivals.is_empty()
    }

    /// День всех соперников по порядку идентификаторов.
    pub fn next_day(
        &mut self,
        content: &Content,
        markets: &mut Markets,
        rng: &mut WorldRng,
        today: GameDate,
    ) {
        let stream = rng.stream(NPC_STREAM);
        for rival in self.rivals.values_mut() {
            rival.live_day(content, markets, stream, today);
        }
    }

    /// Сколько точек этого типа держат в городе соперники, которые в деле.
    pub fn business_points(&self, kind: &BusinessKindId, city: &CityId) -> u32 {
        self.rivals
            .values()
            .filter(|rival| rival.fate().is_active() && rival.home() == city)
            .map(|rival| rival.business_points(kind))
            .sum()
    }

    /// Соперники в деле, подходящие под условие события: стратегия, отношение к игроку и давность
    /// обиды.
    pub fn matching(&self, condition: &RivalCondition, today: GameDate) -> Vec<&NpcId> {
        self.rivals
            .values()
            .filter(|rival| {
                let relation = rival.memory().relation();
                rival.fate().is_active()
                    && condition
                        .strategy
                        .is_none_or(|strategy| strategy == rival.strategy())
                    && condition
                        .relation_at_least
                        .is_none_or(|least| relation >= least)
                    && condition
                        .relation_at_most
                        .is_none_or(|most| relation <= most)
                    && condition.offended_days.is_none_or(|days| {
                        rival
                            .memory()
                            .offended_days(today)
                            .is_some_and(|offended| offended >= days)
                    })
            })
            .map(Rival::npc)
            .collect()
    }
}
