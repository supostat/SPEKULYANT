use std::collections::BTreeMap;

use serde::Deserialize;

use super::Stash;
use crate::content::{EndingId, EpochId, FactionId, FlagId};
use crate::economy::Real;
use crate::time::GameDate;

/// Правила зоны из `data/zone/rules.toml` (решение 15.8): неделя зоны, срок, который не
/// пересидеть, здоровье, авторитет, УДО, передача, выход и доверенные лица, а также зоны эпох.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZoneRules {
    /// Дней в неделе зоны; неделя проходит за время одного дня на воле.
    pub week_days: u32,
    /// Срок длиннее — жизнь кончается приговором.
    pub longest_sentence_months: u32,
    /// Финал за срок, который не пересидеть.
    pub life_ending: EndingId,
    /// Шанс события зоны в неделю.
    pub event_chance: f64,
    /// Шанс в неделю, что зона отнимет здоровье, и сколько делений.
    pub health_loss_chance: f64,
    pub health_loss: i32,
    /// Авторитет снизу вверх: слово каждого уровня.
    pub authority_levels: Vec<String>,
    /// Уровень на первый день срока, от 1.
    pub start_authority: u8,
    /// Шанс в неделю без работы подняться на уровень авторитета.
    pub idle_authority_chance: f64,
    /// Тому, кто поднялся выше этого уровня, УДО не дают.
    pub parole_max_authority: u8,
    /// После отказа в УДО новая просьба — через столько недель.
    pub parole_retry_weeks: u32,
    pub parcel: Parcel,
    /// Флаг, который ставит выход на волю: им открывается «Бывший сиделец».
    pub release_flag: FlagId,
    /// Связи «оттуда»: на выходе репутация этой фракции меняется на значение по уровню авторитета.
    pub ties_faction: FactionId,
    pub ties: Vec<i32>,
    pub trustees: TrusteeRules,
    /// Зона каждой эпохи календаря.
    pub camps: BTreeMap<EpochId, Camp>,
}

/// Передача с воли: цена в рублях 1984 года и что в ней.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Parcel {
    pub price: Real,
    pub contents: Stash,
}

/// Доверенные лица (§12): раз в месяц отсидки — шанс исчезнуть с выручкой по делению
/// лояльности и шанс воровать по делению честности, и какую долю выручки уносит ворующий.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrusteeRules {
    pub vanish_chances: Vec<f64>,
    pub steal_chances: Vec<f64>,
    pub steal_share: f64,
}

/// Зона эпохи: название, сколько сигарет в неделю платит промзона, раз во сколько недель
/// разрешена передача, после какой доли срока можно просить об УДО и с каким шансом его дают при
/// полной трудовой характеристике, и сколько стоит выход за взятку, если его продают.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Camp {
    pub title: String,
    pub work_pay: u32,
    pub parcel_weeks: u32,
    pub parole_after: f64,
    pub parole_chance: f64,
    pub bribe: Option<Real>,
}

impl ZoneRules {
    /// Зона эпохи этого дня; у каждой эпохи календаря она есть — это проверяет загрузка.
    pub fn camp_on<'a>(&'a self, epoch: &str) -> &'a Camp {
        self.camps
            .iter()
            .find(|(id, _)| id.as_str() == epoch)
            .map(|(_, camp)| camp)
            .expect("у каждой эпохи календаря есть зона")
    }

    /// Самый высокий уровень авторитета.
    pub fn top_authority(&self) -> u8 {
        u8::try_from(self.authority_levels.len()).expect("уровней авторитета меньше 256")
    }

    /// Слово уровня авторитета.
    pub fn authority_title(&self, authority: u8) -> &str {
        &self.authority_levels[usize::from(authority) - 1]
    }

    /// День через `weeks` недель зоны.
    pub fn weeks_after(&self, date: GameDate, weeks: u32) -> GameDate {
        date.after_days(weeks * self.week_days)
    }
}
