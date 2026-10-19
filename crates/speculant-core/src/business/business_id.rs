use std::fmt;

use crate::content::{BusinessKindId, CityId};

const SEPARATOR: char = '@';

/// Дело игрока — тип в городе: вторая точка того же типа в том же городе расширяет дело, а не
/// открывает новое. В командах и снимке пишется «тип@город»: `kiosks@moscow`.
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
