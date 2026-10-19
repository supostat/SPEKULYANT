//! Контент как данные (§10, §14.1): товары, города, маршруты, эпохи, правила толкучки, индекс цен
//! и курсы, места хранения денег, фракции и правила репутации, биографии, жильё, финалы и правила
//! жизни, шаблоны NPC, дела — формы, типы, политики, привычки и правила, — события и правила их
//! розыгрыша, правила зоны и её услуги, исторические узлы, шаблоны генератора песочницы и мелодия
//! заставки. Формат — TOML, файл на сущность в `data/`, имя файла — идентификатор (решение 15.7).
//! Загрузка проверяет схему и ссылки и возвращает все ошибки сразу, каждая — «файл:строка».

mod biography;
mod business_form;
mod business_kind;
mod city;
mod conditions;
mod consequences;
mod content_error;
mod content_file;
mod employee_traits;
mod ending;
mod epoch_profile;
mod event;
mod faction;
mod good;
mod habit;
mod history_node;
mod history_outcome;
mod history_template;
mod housing;
mod ids;
mod loader;
mod melody;
mod money_place;
mod npc_template;
mod policy;
mod road_event;
mod route;
mod scale_range;
mod validation;
mod zone_service;

use std::collections::BTreeMap;
use std::path::Path;

pub use biography::{AgeRange, Biography};
pub use business_form::BusinessForm;
pub use business_kind::BusinessKind;
pub use city::City;
pub use conditions::{
    AuthorityCondition, Conditions, DateWindow, GoodCondition, MoneyCondition, ReputationCondition,
    RivalCondition,
};
pub use consequences::{
    BusinessChange, Consequences, ContactChange, GoodChange, PolicySetting, PriceShock,
    ReputationChange, RivalChange, ZoneChange,
};
pub use content_error::ContentError;
pub use content_file::ContentFile;
pub use employee_traits::EmployeeTraits;
pub use ending::Ending;
pub use epoch_profile::{EpochProfile, Threat};
pub use event::{Choice, Event, FollowUp};
pub use faction::Faction;
pub use good::Good;
pub use habit::Habit;
pub use history_node::{HistoryNode, HistoryWindow};
pub use history_outcome::HistoryOutcome;
pub use history_template::HistoryTemplate;
pub use housing::Housing;
pub use ids::{
    BiographyId, BusinessFormId, BusinessKindId, CityId, EndingId, EpochId, EventId, FactionId,
    FlagId, GoodId, HabitId, HousingId, MoneyPlaceId, NpcId, TimelineNodeId, ZoneServiceId,
};
pub use melody::{Melody, Note, Pitch};
pub use money_place::{InterestPoint, MoneyPlace};
pub use npc_template::NpcTemplate;
pub use policy::{Policy, PolicyOption};
pub use road_event::RoadEvent;
pub use route::Route;
pub use scale_range::ScaleRange;
pub use zone_service::ZoneService;

use crate::business::{BusinessRules, PolicyKind};
use crate::economy::{MarketRules, MoneyRules};
use crate::events::EventRules;
use crate::history::HistoryGenerator;
use crate::life::LifeRules;
use crate::npc::RivalRules;
use crate::reputation::ReputationRules;
use crate::time::{GameDate, TimeRules};
use crate::zone::{Camp, ZoneRules};

#[derive(Clone, Debug)]
pub struct Content {
    pub time: TimeRules,
    pub market: MarketRules,
    pub money: MoneyRules,
    pub reputation: ReputationRules,
    pub event_rules: EventRules,
    pub rival_rules: RivalRules,
    pub business: BusinessRules,
    pub life: LifeRules,
    pub zone: ZoneRules,
    pub history_generator: HistoryGenerator,
    /// Мелодия заставки для пищалки PC-speaker.
    pub boot_melody: Melody,
    pub epochs: BTreeMap<EpochId, EpochProfile>,
    pub goods: BTreeMap<GoodId, Good>,
    pub cities: BTreeMap<CityId, City>,
    pub routes: Vec<Route>,
    pub money_places: BTreeMap<MoneyPlaceId, MoneyPlace>,
    pub factions: BTreeMap<FactionId, Faction>,
    pub biographies: BTreeMap<BiographyId, Biography>,
    pub housing: BTreeMap<HousingId, Housing>,
    pub endings: BTreeMap<EndingId, Ending>,
    pub npcs: BTreeMap<NpcId, NpcTemplate>,
    pub business_forms: BTreeMap<BusinessFormId, BusinessForm>,
    pub business_kinds: BTreeMap<BusinessKindId, BusinessKind>,
    pub policies: BTreeMap<PolicyKind, Policy>,
    pub habits: BTreeMap<HabitId, Habit>,
    pub events: BTreeMap<EventId, Event>,
    pub zone_services: BTreeMap<ZoneServiceId, ZoneService>,
    pub history: BTreeMap<TimelineNodeId, HistoryNode>,
    pub history_templates: BTreeMap<TimelineNodeId, HistoryTemplate>,
    /// Отпечаток файлов данных: файл жизни, сделанный с другими данными, не загрузится.
    pub fingerprint: u64,
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

    /// Зона эпохи, в которой лежит день; у каждой эпохи календаря она есть — это проверяет загрузка.
    pub fn camp_on(&self, date: GameDate) -> &Camp {
        self.zone.camp_on(&self.time.epochs.epoch_on(date).id)
    }
}
