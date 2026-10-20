use crate::content::{Content, Route};
use crate::time::GameDate;

/// Можно ли сейчас ехать по маршруту (§4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
    /// Город открыт — ехать может каждый.
    Open,
    /// Пока только по блату: через знакомого, за услугу (ГДР и Венгрия до 1990).
    ByConnections,
    /// Город ещё закрыт (Польша до 1990–91, Китай до 1992).
    Closed,
}

impl Access {
    pub fn of(route: &Route, content: &Content, date: GameDate) -> Self {
        let city = &content.cities[&route.to];
        if !city.is_open_on(date) {
            Self::Closed
        } else if city
            .connections_only_until
            .is_some_and(|until| date < until)
        {
            Self::ByConnections
        } else {
            Self::Open
        }
    }
}
