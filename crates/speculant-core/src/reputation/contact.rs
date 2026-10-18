use super::{Availability, ContactRefusal, ReputationRules};
use crate::content::{ContactChange, NpcId};
use crate::time::GameDate;

/// Связь (§9): конкретный человек из шаблонов NPC. Отношение — расположение к игроку от поступков,
/// долг — счёт услуг: больше нуля — он должен игроку, меньше — игрок ему. История — что было
/// между ними, по датам.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contact {
    npc: NpcId,
    relation: i32,
    debt: i32,
    history: Vec<String>,
    availability: Availability,
}

impl Contact {
    /// Знакомый без истории: ровное отношение, никто никому не должен.
    pub fn new(npc: NpcId) -> Self {
        Self {
            npc,
            relation: 0,
            debt: 0,
            history: Vec::new(),
            availability: Availability::Available,
        }
    }

    pub fn npc(&self) -> &NpcId {
        &self.npc
    }

    pub fn relation(&self) -> i32 {
        self.relation
    }

    pub fn debt(&self) -> i32 {
        self.debt
    }

    pub fn history(&self) -> &[String] {
        &self.history
    }

    pub fn availability(&self) -> Availability {
        self.availability
    }

    pub fn is_lost(&self) -> bool {
        self.availability == Availability::Lost
    }

    /// До человека можно достучаться: он не потерян и не в отъезде.
    pub fn is_available_on(&self, today: GameDate) -> bool {
        match self.availability {
            Availability::Available => true,
            Availability::AwayUntil(until) => today >= until,
            Availability::Lost => false,
        }
    }

    /// Попросить об услуге: помогает, если он на связи, не обижен и за игроком меньше
    /// `max_debt` услуг; после этого игрок ему должен ещё одну.
    pub fn ask(
        &mut self,
        rules: ReputationRules,
        today: GameDate,
        what: &str,
    ) -> Result<(), ContactRefusal> {
        match self.availability {
            Availability::Lost => return Err(ContactRefusal::Lost),
            Availability::AwayUntil(until) if today < until => {
                return Err(ContactRefusal::Away { until });
            }
            Availability::AwayUntil(_) | Availability::Available => {}
        }
        if self.relation < 0 {
            return Err(ContactRefusal::Offended);
        }
        if self.owed_favours() >= rules.max_debt {
            return Err(ContactRefusal::OwedTooMuch);
        }
        self.availability = Availability::Available;
        self.debt -= 1;
        self.record(today, what);
        Ok(())
    }

    /// Задолжать: взять деньгами, товаром или делом без просьбы — он дал сам, долг растёт.
    pub fn owe(&mut self, favours: u32, today: GameDate, what: &str) {
        self.debt = self.debt.saturating_sub_unsigned(favours);
        self.record(today, what);
    }

    /// Оказать услугу или вернуть долг: теперь он должен игроку больше.
    pub fn help(&mut self, favours: u32, today: GameDate, what: &str) {
        self.debt = self.debt.saturating_add_unsigned(favours);
        self.record(today, what);
    }

    /// Отношение от поступков — в пределах правил.
    pub fn change_relation(
        &mut self,
        change: i32,
        rules: ReputationRules,
        today: GameDate,
        what: &str,
    ) {
        self.relation = rules.clamp(self.relation.saturating_add(change));
        self.record(today, what);
    }

    /// Уехал, лёг в больницу, сел — до даты.
    pub fn send_away(&mut self, until: GameDate, today: GameDate, what: &str) {
        self.availability = Availability::AwayUntil(until);
        self.record(today, what);
    }

    /// Потерять навсегда.
    pub fn lose(&mut self, today: GameDate, what: &str) {
        self.availability = Availability::Lost;
        self.record(today, what);
    }

    /// Последствие события (§10.1): отношение, услуги, отъезд и потеря разом, одной записью в истории.
    pub fn apply(
        &mut self,
        change: &ContactChange,
        rules: ReputationRules,
        today: GameDate,
        what: &str,
    ) {
        self.relation = rules.clamp(self.relation.saturating_add(change.relation));
        self.debt = self.debt.saturating_add(change.favours);
        if let Some(days) = change.away_days {
            self.availability = Availability::AwayUntil(today.after_days(days));
        }
        if change.lost {
            self.availability = Availability::Lost;
        }
        self.record(today, what);
    }

    /// Сколько услуг игрок ему должен.
    fn owed_favours(&self) -> u32 {
        self.debt.min(0).unsigned_abs()
    }

    fn record(&mut self, today: GameDate, what: &str) {
        self.history.push(format!("{}: {what}", today.numeric()));
    }
}
