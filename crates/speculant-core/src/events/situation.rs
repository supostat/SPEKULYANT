use std::collections::{BTreeMap, BTreeSet};

use crate::content::{Choice, CityId, Conditions, Consequences, Content, Event, FlagId, NpcId};
use crate::economy::{Currency, Money, Real, Trader};
use crate::reputation::{Contact, FactionReputation};
use crate::time::GameDate;

/// Положение дел, над которым считаются условия событий (§10.1): день, город (в пути — город
/// отправления), дорога, деньги и карман, репутация, связи и флаги прошлых решений.
pub struct Situation<'a> {
    pub content: &'a Content,
    pub today: GameDate,
    pub city: &'a CityId,
    pub on_road: bool,
    pub player: &'a Trader,
    pub reputation: &'a FactionReputation,
    pub contacts: &'a BTreeMap<NpcId, Contact>,
    pub flags: &'a BTreeSet<FlagId>,
}

impl Situation<'_> {
    /// Событие может случиться: оно из каталога текущей эпохи, дорожное — только в пути, обычное —
    /// только не в пути, и его условия выполнены. События зоны (без эпохи) в обычной жизни не
    /// случаются.
    pub fn allows_event(&self, event: &Event) -> bool {
        let epoch = &self.content.time.epochs.epoch_on(self.today).id;
        event
            .epoch
            .as_ref()
            .is_some_and(|own| own.as_str() == epoch)
            && event.road == self.on_road
            && self.allows(&event.conditions)
    }

    /// Выбор доступен: его условия выполнены, и у игрока есть деньги и товар, которые он отдаёт.
    pub fn allows_choice(&self, choice: &Choice) -> bool {
        self.allows(&choice.conditions) && self.affords(&choice.consequences)
    }

    /// Условия выполнены; пустые условия выполняются всегда. Окно дат включает `from` и не
    /// включает `until`. Узлы таймлайна появятся с генератором истории, до тех пор условие
    /// `timeline` не выполняется.
    pub fn allows(&self, conditions: &Conditions) -> bool {
        let epoch = &self.content.time.epochs.epoch_on(self.today).id;
        let cash = self.player.money().cash();
        let in_epoch = conditions.epochs.is_empty()
            || conditions
                .epochs
                .iter()
                .any(|allowed| allowed.as_str() == epoch);
        let in_window = conditions.window.is_none_or(|window| {
            window.from <= self.today && window.until.is_none_or(|until| self.today < until)
        });
        let in_city = conditions.cities.is_empty() || conditions.cities.contains(self.city);
        let reputation_fits = conditions.reputation.iter().all(|bounds| {
            let level = self.reputation.level(&bounds.faction);
            bounds.at_least.is_none_or(|least| level >= least)
                && bounds.at_most.is_none_or(|most| level <= most)
        });
        let has_money = conditions
            .money
            .is_none_or(|money| self.content.money.real_value(cash, self.today) >= money.at_least);
        let has_goods = conditions
            .goods
            .iter()
            .all(|needed| self.player.pocket().quantity(&needed.good) >= needed.at_least);
        let flags_fit = conditions
            .flags_set
            .iter()
            .all(|flag| self.flags.contains(flag))
            && !conditions
                .flags_unset
                .iter()
                .any(|flag| self.flags.contains(flag));
        let npc_reachable = conditions
            .npc
            .as_ref()
            .is_none_or(|npc| self.is_reachable(npc));
        in_epoch
            && in_window
            && in_city
            && reputation_fits
            && has_money
            && has_goods
            && flags_fit
            && npc_reachable
            && conditions.timeline.is_none()
    }

    /// Деньги, которые выбор забирает, есть наличными, а товар, который он забирает, — в кармане.
    fn affords(&self, consequences: &Consequences) -> bool {
        let pays = consequences.money.is_none_or(|money| {
            !money.is_negative() || self.player.money().cash().covers(self.cost(money))
        });
        let gives_goods = consequences.goods.iter().all(|change| {
            change.change >= 0
                || self.player.pocket().quantity(&change.good) >= change.change.unsigned_abs()
        });
        pays && gives_goods
    }

    fn cost(&self, money: Real) -> Money {
        let amount = Real::from_kopecks(money.kopecks().abs());
        self.content
            .money
            .from_real(amount, Currency::Ruble, self.today)
    }

    /// С человеком можно встретиться: он не потерян и не в отъезде; незнакомый — тоже можно.
    fn is_reachable(&self, npc: &NpcId) -> bool {
        self.contacts
            .get(npc)
            .is_none_or(|contact| contact.is_available_on(self.today))
    }
}
