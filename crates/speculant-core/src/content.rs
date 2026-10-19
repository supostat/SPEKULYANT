//! Контент как данные (§10, §14.1): товары, города, маршруты, эпохи, правила толкучки, индекс цен
//! и курсы, места хранения денег, фракции и правила репутации, биографии, шаблоны NPC, события и
//! правила их розыгрыша. Формат — TOML, файл на
//! сущность в `data/`, имя файла — идентификатор (решение 15.7).
//! Загрузка проверяет схему и ссылки и возвращает все ошибки сразу, каждая — «файл:строка».

mod biography;
mod city;
mod conditions;
mod consequences;
mod content_error;
mod content_file;
mod epoch_profile;
mod event;
mod faction;
mod good;
mod ids;
mod loader;
mod money_place;
mod npc_template;
mod road_event;
mod route;
mod validation;

use std::collections::BTreeMap;
use std::path::Path;

pub use biography::Biography;
pub use city::City;
pub use conditions::{
    Conditions, DateWindow, GoodCondition, MoneyCondition, ReputationCondition, RivalCondition,
};
pub use consequences::{
    Consequences, ContactChange, GoodChange, PriceShock, ReputationChange, RivalChange,
};
pub use content_error::ContentError;
pub use content_file::ContentFile;
pub use epoch_profile::{EpochProfile, Threat};
pub use event::{Choice, Event, FollowUp};
pub use faction::Faction;
pub use good::Good;
pub use ids::{
    BiographyId, CityId, EpochId, EventId, FactionId, FlagId, GoodId, MoneyPlaceId, NpcId,
    TimelineNodeId,
};
pub use money_place::{InterestPoint, MoneyPlace};
pub use npc_template::NpcTemplate;
pub use road_event::RoadEvent;
pub use route::Route;

use crate::economy::{MarketRules, MoneyRules};
use crate::events::EventRules;
use crate::npc::RivalRules;
use crate::reputation::ReputationRules;
use crate::time::{GameDate, TimeRules};

#[derive(Clone, Debug)]
pub struct Content {
    pub time: TimeRules,
    pub market: MarketRules,
    pub money: MoneyRules,
    pub reputation: ReputationRules,
    pub event_rules: EventRules,
    pub rival_rules: RivalRules,
    pub epochs: BTreeMap<EpochId, EpochProfile>,
    pub goods: BTreeMap<GoodId, Good>,
    pub cities: BTreeMap<CityId, City>,
    pub routes: Vec<Route>,
    pub money_places: BTreeMap<MoneyPlaceId, MoneyPlace>,
    pub factions: BTreeMap<FactionId, Faction>,
    pub biographies: BTreeMap<BiographyId, Biography>,
    pub npcs: BTreeMap<NpcId, NpcTemplate>,
    pub events: BTreeMap<EventId, Event>,
}

impl Content {
    /// Читает каталог данных с диска — так работают симулятор и валидатор.
    pub fn load(dir: &Path) -> Result<Self, Vec<ContentError>> {
        let files = Self::read_files(dir)?;
        Self::from_files(files)
    }

    /// Файлы каталога данных с путями относительно его корня — для сборки контента в памяти.
    pub fn read_files(dir: &Path) -> Result<Vec<ContentFile>, Vec<ContentError>> {
        content_file::read_tree(dir)
    }

    /// Собирает контент из файлов в памяти — так его получает мост, который вшивает `data/` при сборке.
    pub fn from_files(files: Vec<ContentFile>) -> Result<Self, Vec<ContentError>> {
        loader::load(files)
    }

    /// Описание эпохи, в которой лежит день; у каждой эпохи календаря оно есть — это проверяет загрузка.
    pub fn epoch_profile_on(&self, date: GameDate) -> &EpochProfile {
        let epoch = self.time.epochs.epoch_on(date);
        self.epochs
            .get(&EpochId::new(epoch.id.as_str()))
            .expect("у каждой эпохи календаря есть описание")
    }
}
