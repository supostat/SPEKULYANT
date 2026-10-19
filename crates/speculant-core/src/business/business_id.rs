use std::fmt;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::content::{BusinessKindId, CityId};

const SEPARATOR: char = '@';

/// Дело игрока — тип в городе: вторая точка того же типа в том же городе расширяет дело, а не
/// открывает новое. В командах, снимке и сохранениях пишется «тип@город»: `kiosks@moscow`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BusinessId {
    pub kind: BusinessKindId,
    pub city: CityId,
}

impl BusinessId {
    pub fn new(kind: BusinessKindId, city: CityId) -> Self {
        Self { kind, city }
    }

    /// Идентификатор из команды; `None`, если строка не «тип@город».
    pub fn parse(text: &str) -> Option<Self> {
        let (kind, city) = text.split_once(SEPARATOR)?;
        (!kind.is_empty() && !city.is_empty())
            .then(|| Self::new(BusinessKindId::new(kind), CityId::new(city)))
    }
}

impl fmt::Display for BusinessId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}{SEPARATOR}{}", self.kind, self.city)
    }
}

impl Serialize for BusinessId {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for BusinessId {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::parse(&text)
            .ok_or_else(|| D::Error::custom(format!("«{text}» — не дело «тип@город»")))
    }
}
