//! Зона (§12, решение 15.8): отдельный режим с недельным тиком — срок в настоящем календаре,
//! сигареты и чай, промзона, передачи и услуги, свой авторитет, доверенные лица на воле и выход
//! по сроку, по УДО, по амнистии и за взятку.

mod release;
mod sentence;
mod stash;
mod trustee_conduct;
mod zone_rules;

pub use release::Release;
pub use sentence::Sentence;
pub use stash::Stash;
pub use trustee_conduct::TrusteeConduct;
pub use zone_rules::{Camp, Parcel, TrusteeRules, ZoneRules};

use rand_chacha::ChaCha8Rng;

use crate::events::EventEngine;
use crate::time::GameDate;

/// На столько уровней за раз растёт авторитет того, кто не работает.
const AUTHORITY_STEP: i32 = 1;
const LOWEST_AUTHORITY: i32 = 1;

/// Отсидка: срок, авторитет, сигареты и чай, работа на промзоне с трудовой характеристикой и
/// когда можно следующую передачу и следующую просьбу об УДО.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Zone {
    sentence: Sentence,
    authority: u8,
    stash: Stash,
    working: bool,
    weeks_served: u32,
    weeks_worked: u32,
    next_parcel: GameDate,
    next_parole_request: GameDate,
}

impl Zone {
    /// Первый день срока: авторитет — стартовый правил, ни сигарет, ни чая, на промзону не
    /// ходит, передачу можно сразу.
    pub fn new(sentence: Sentence, rules: &ZoneRules) -> Self {
        Self {
            sentence,
            authority: rules.start_authority,
            stash: Stash::default(),
            working: false,
            weeks_served: 0,
            weeks_worked: 0,
            next_parcel: sentence.from(),
            next_parole_request: sentence.from(),
        }
    }

    pub fn sentence(&self) -> &Sentence {
        &self.sentence
    }

    pub fn authority(&self) -> u8 {
        self.authority
    }

    pub fn stash(&self) -> Stash {
        self.stash
    }

    pub fn is_working(&self) -> bool {
        self.working
    }

    pub fn set_working(&mut self, working: bool) {
        self.working = working;
    }

    pub fn next_parcel(&self) -> GameDate {
        self.next_parcel
    }

    /// Неделя прошла: промзона платит сигаретами зоны эпохи и пишет в трудовую характеристику, а
    /// кто не работает, с шансом правил поднимается в авторитете.
    pub fn live_week(&mut self, rules: &ZoneRules, camp: &Camp, rng: &mut ChaCha8Rng) {
        self.weeks_served += 1;
        if self.working {
            self.weeks_worked += 1;
            self.receive(Stash {
                cigarettes: camp.work_pay,
                ..Stash::default()
            });
        } else if EventEngine::roll(rng, rules.idle_authority_chance) {
            self.change_authority(AUTHORITY_STEP, rules);
        }
    }

    /// Авторитет в пределах шкалы правил.
    pub fn change_authority(&mut self, change: i32, rules: &ZoneRules) {
        let level = (i32::from(self.authority) + change)
            .clamp(LOWEST_AUTHORITY, i32::from(rules.top_authority()));
        self.authority = u8::try_from(level).expect("уровней авторитета меньше 256");
    }

    pub fn receive(&mut self, stash: Stash) {
        self.stash = self.stash.plus(stash);
    }

    /// Платит сигаретами и чаем; хватает ли, проверяет вызывающий.
    pub fn pay(&mut self, price: Stash) {
        self.stash = self.stash.minus(price);
    }

    /// Передача пришла; следующую зона эпохи разрешит с `next`.
    pub fn receive_parcel(&mut self, contents: Stash, next: GameDate) {
        self.receive(contents);
        self.next_parcel = next;
    }

    /// Трудовая характеристика — доля отработанных недель; до конца первой недели она пуста.
    pub fn work_record(&self) -> f64 {
        if self.weeks_served == 0 {
            return 0.0;
        }
        f64::from(self.weeks_worked) / f64::from(self.weeks_served)
    }

    /// С какого дня можно просить об УДО: отсижена доля срока, которую требует зона эпохи, и после
    /// отказа прошёл срок правил.
    pub fn parole_from(&self, camp: &Camp) -> GameDate {
        self.sentence
            .share_served_on(camp.parole_after)
            .max(self.next_parole_request)
    }

    /// Шанс УДО: шанс зоны эпохи, умноженный на трудовую характеристику; тому, кто поднялся в
    /// авторитете выше порога правил, УДО не дают.
    pub fn parole_chance(&self, rules: &ZoneRules, camp: &Camp) -> f64 {
        if self.authority > rules.parole_max_authority {
            return 0.0;
        }
        camp.parole_chance * self.work_record()
    }

    /// В УДО отказали; следующая просьба — с `next_request`.
    pub fn refuse_parole(&mut self, next_request: GameDate) {
        self.next_parole_request = next_request;
    }

    /// Суд добавил к сроку месяцы или скостил их.
    pub fn change_term(&mut self, months: i32) {
        self.sentence.change(months);
    }
}
