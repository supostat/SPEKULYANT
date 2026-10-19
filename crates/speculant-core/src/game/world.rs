//! Мир одной жизни — всё, что ложится в файл жизни (§11.4).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::business::{Business, BusinessId};
use crate::content::{CityId, NpcId};
use crate::economy::{Markets, Trader};
use crate::events::EventEngine;
use crate::history::Timeline;
use crate::life::{Character, Chronicle, LifeSummary};
use crate::npc::Rivals;
use crate::reputation::{Contact, FactionReputation};
use crate::rng::WorldRng;
use crate::time::Clock;
use crate::travel::Journey;
use crate::view::Screen;
use crate::zone::Zone;

/// Мир жизни: режим, персонаж и летопись, итоги, часы и случай, история, город, рынки, кошелёк,
/// репутация, связи, соперники, дела, события, дорога, зона, лента и открытый экран.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub(super) struct World {
    /// Ironman-жизнь сохраняется сама и не откатывается.
    pub(super) ironman: bool,
    pub(super) character: Character,
    pub(super) chronicle: Chronicle,
    /// Итоги, когда жизнь кончилась; до тех пор — `None`.
    pub(super) summary: Option<LifeSummary>,
    pub(super) clock: Clock,
    pub(super) rng: WorldRng,
    /// История мира, зафиксированная по сиду при его создании.
    pub(super) timeline: Timeline,
    pub(super) city: CityId,
    pub(super) markets: Markets,
    pub(super) player: Trader,
    pub(super) reputation: FactionReputation,
    pub(super) contacts: BTreeMap<NpcId, Contact>,
    pub(super) rivals: Rivals,
    pub(super) businesses: BTreeMap<BusinessId, Business>,
    pub(super) events: EventEngine,
    pub(super) journey: Option<Journey>,
    /// Отсидка, пока игрок в зоне.
    pub(super) zone: Option<Zone>,
    pub(super) messages: Vec<String>,
    pub(super) screen: Screen,
}
