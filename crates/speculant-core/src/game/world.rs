//! Мир одной жизни — всё, что ложится в файл жизни (§11.4).

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::business::{Business, BusinessId};
use crate::content::{CityId, NpcId};
use crate::economy::{Markets, PriceHistory, SeenPrices, Trader};
use crate::events::EventEngine;
use crate::history::Timeline;
use crate::life::{Character, Chronicle, LifeSummary, Pager};
use crate::npc::Rivals;
use crate::reputation::{Contact, FactionReputation};
use crate::rng::WorldRng;
use crate::time::Clock;
use crate::travel::Journey;
use crate::view::Screen;
use crate::zone::Zone;

/// Мир жизни: режим, персонаж и летопись, итоги, часы и случай, история, город, рынки, история их
/// цен и цены, какими их видел игрок, кошелёк, репутация, связи, соперники, дела, события, дорога,
/// зона, лента, пейджер и открытый экран.
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
    /// Цены толкучек раз в неделю за последний год — для графика цены.
    pub(super) price_history: PriceHistory,
    /// Толкучки, какими игрок видел их в день отъезда, — для цен по городам.
    pub(super) seen_prices: SeenPrices,
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
    pub(super) pager: Pager,
    pub(super) screen: Screen,
}
