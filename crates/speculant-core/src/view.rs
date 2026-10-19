//! Снимок мира для интерфейса: всё, что показывают одиннадцать экранов §13.3, строками, готовыми
//! к показу. Деньги уже в номинале текущей валюты, даты — по-русски; интерфейс ничего не считает.

mod biography;
mod business;
mod character;
mod contact;
mod event;
mod faction;
mod header;
mod level;
mod lobby;
mod market;
mod money;
mod notebook;
mod pager;
mod pocket;
mod records;
mod screen;
mod summary;
mod time;
mod travel;
mod zone;

pub use biography::BiographyView;
pub use business::{
    BusinessKindView, BusinessView, CandidateView, ForecastView, ManagerView, PolicyView,
    ReportView,
};
pub use character::{CharacterView, ExitView, HomeView};
pub use contact::ContactView;
pub use event::{ChoiceView, EventView, OutcomeView};
pub use faction::FactionView;
pub use header::HeaderView;
pub use level::LevelView;
pub use lobby::LobbyView;
pub use market::{MarketRowView, MarketView};
pub use money::{MoneyPlaceView, MoneyView};
pub use notebook::NotebookEntryView;
pub use pager::{PagerMessageView, PagerView};
pub use pocket::{PocketItemView, PocketView};
pub use records::{RecordView, RecordsView};
pub use screen::Screen;
pub use summary::{LifeSummaryView, SummaryLineView};
pub use time::TimeView;
pub use travel::{DestinationView, JourneyView, TravelView};
pub use zone::{ZoneOfferView, ZoneServiceView, ZoneView};

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateView {
    pub screen: Screen,
    /// Ironman-жизнь сохраняется сама: меню не предлагает ни сохранения, ни загрузки.
    pub ironman: bool,
    pub header: HeaderView,
    pub character: CharacterView,
    pub homes: Vec<HomeView>,
    pub exits: Vec<ExitView>,
    pub time: TimeView,
    pub market: MarketView,
    pub travel: TravelView,
    pub pocket: PocketView,
    pub rumors: Vec<String>,
    pub messages: Vec<String>,
    pub pending_event: Option<EventView>,
    pub pager: Option<PagerView>,
    pub businesses: Vec<BusinessView>,
    pub candidates: Vec<CandidateView>,
    pub business_kinds: Vec<BusinessKindView>,
    pub contacts: Vec<ContactView>,
    pub factions: Vec<FactionView>,
    pub money: MoneyView,
    pub notebook: Vec<NotebookEntryView>,
    pub records: RecordsView,
    pub zone: Option<ZoneView>,
    pub summary: Option<LifeSummaryView>,
    pub biographies: Vec<BiographyView>,
}
