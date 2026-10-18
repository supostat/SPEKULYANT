//! Снимок мира для интерфейса: всё, что показывают одиннадцать экранов §13.3, строками, готовыми
//! к показу. Деньги уже в номинале текущей валюты, даты — по-русски; интерфейс ничего не считает.

mod biography;
mod business;
mod contact;
mod event;
mod faction;
mod header;
mod level;
mod market;
mod money;
mod notebook;
mod pager;
mod pocket;
mod screen;
mod summary;
mod time;
mod zone;

pub use biography::BiographyView;
pub use business::{BusinessView, ForecastView, ManagerView, PolicyView};
pub use contact::ContactView;
pub use event::{ChoiceView, EventView, OutcomeView};
pub use faction::FactionView;
pub use header::HeaderView;
pub use level::LevelView;
pub use market::{MarketRowView, MarketView};
pub use money::{MoneyPlaceView, MoneyView};
pub use notebook::NotebookEntryView;
pub use pager::{PagerMessageView, PagerView};
pub use pocket::{PocketItemView, PocketView};
pub use screen::Screen;
pub use summary::{LifeSummaryView, SummaryLineView};
pub use time::TimeView;
pub use zone::ZoneView;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateView {
    pub screen: Screen,
    pub header: HeaderView,
    pub time: TimeView,
    pub market: MarketView,
    pub pocket: PocketView,
    pub rumors: Vec<String>,
    pub messages: Vec<String>,
    pub pending_event: Option<EventView>,
    pub pager: Option<PagerView>,
    pub businesses: Vec<BusinessView>,
    pub manager: Option<ManagerView>,
    pub contacts: Vec<ContactView>,
    pub factions: Vec<FactionView>,
    pub money: MoneyView,
    pub notebook: Vec<NotebookEntryView>,
    pub zone: Option<ZoneView>,
    pub summary: Option<LifeSummaryView>,
    pub biographies: Vec<BiographyView>,
}
