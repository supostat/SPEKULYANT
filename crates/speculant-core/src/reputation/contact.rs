use super::{Availability, ContactRefusal, ReputationRules};
use crate::content::{ContactChange, NpcId};
use crate::time::GameDate;

/// Связь (§9): конкретный человек из шаблонов NPC. Отношение — расположение к игроку от поступков,
/// долг — счёт услуг: больше нуля — он должен игроку, меньше — игрок ему. Обида помнится с того
/// дня, как отношение ушло в минус (§8). История — что было между ними, по датам.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Contact {
    npc: NpcId,
    relation: i32,
    offended_since: Option<GameDate>,
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
            offended_since: None,
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

    /// Сколько дней длится обида; `None`, если её нет.
    pub fn offended_days(&self, today: GameDate) -> Option<u32> {
        self.offended_since.map(|since| {
            u32::try_from(today.day_index() - since.day_index())
                .expect("обида началась не позже сегодняшнего дня")
        })
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
        if let Some(refusal) = self.refusal(rules, today) {
            return Err(refusal);
        }
        self.availability = Availability::Available;
        self.debt -= 1;
        self.record(today, what);
        Ok(())
    }

    /// Почему он сейчас не поможет; `None` — поможет.
    pub fn refusal(&self, rules: ReputationRules, today: GameDate) -> Option<ContactRefusal> {
        match self.availability {
            Availability::Lost => return Some(ContactRefusal::Lost),
            Availability::AwayUntil(until) if today < until => {
                return Some(ContactRefusal::Away { until });
            }
            Availability::AwayUntil(_) | Availability::Available => {}
        }
        if self.relation < 0 {
            Some(ContactRefusal::Offended)
        } else if self.owed_favours() >= rules.max_debt {
            Some(ContactRefusal::OwedTooMuch)
        } else {
            None
        }
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
        self.set_relation(rules.clamp(self.relation.saturating_add(change)), today);
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
        self.set_relation(
            rules.clamp(self.relation.saturating_add(change.relation)),
            today,
        );
        self.debt = self.debt.saturating_add(change.favours);
        if let Some(days) = change.away_days {
            self.availability = Availability::AwayUntil(today.after_days(days));
        }
        if change.lost {
            self.availability = Availability::Lost;
        }
        self.record(today, what);
    }

    /// Новое отношение: обида начинается в день, когда оно ушло в минус, и кончается с примирением.
    fn set_relation(&mut self, relation: i32, today: GameDate) {
        self.relation = relation;
        if relation >= 0 {
            self.offended_since = None;
        } else if self.offended_since.is_none() {
            self.offended_since = Some(today);
        }
    }

    /// Сколько услуг игрок ему должен.
    fn owed_favours(&self) -> u32 {
        self.debt.min(0).unsigned_abs()
    }

    fn record(&mut self, today: GameDate, what: &str) {
        self.history.push(format!("{}: {what}", today.numeric()));
    }
}
