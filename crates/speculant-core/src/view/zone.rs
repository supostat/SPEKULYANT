use serde::{Deserialize, Serialize};

use super::{LevelView, PocketItemView};

/// Зона: где сидит, срок и день выхода по сроку, авторитет, сигареты и чай, промзона, услуги,
/// передача, УДО, взятка, если зона эпохи её берёт, и доверенные лица на воле.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneView {
    pub camp: String,
    pub sentence: String,
    pub release: String,
    pub authority: LevelView,
    pub stash: Vec<PocketItemView>,
    pub working: bool,
    pub services: Vec<ZoneServiceView>,
    pub parcel: ZoneOfferView,
    pub parole: ZoneOfferView,
    pub bribe: Option<ZoneOfferView>,
    pub trustees: Vec<String>,
}

/// Услуга зоны: цена в пачках и хватает ли запаса.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneServiceView {
    pub service: String,
    pub title: String,
    pub price: String,
    pub available: bool,
}

/// Передача, УДО или взятка: цена или день, с которого можно, и можно ли сейчас.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ZoneOfferView {
    pub terms: String,
    pub available: bool,
}
