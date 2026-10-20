use serde::{Deserialize, Serialize};

/// Экран «Поездка» (§13.3 п. 3): куда можно ехать отсюда, а в пути — откуда, куда и когда приезд.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TravelView {
    pub journey: Option<JourneyView>,
    pub destinations: Vec<DestinationView>,
}

/// Направление: город, дни в пути, стоимость в номинале дня; по блату — через знакомого.
/// Недоступно, если не хватает денег или по блату помочь некому. Категории товаров, которые там
/// дешевле или дороже, чем здесь, — по множителям городов, если разница окупает разницу цен
/// покупки и продажи; названия — по алфавиту.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DestinationView {
    pub city: String,
    pub name: String,
    pub days: u32,
    pub fare: String,
    pub by_connections: bool,
    pub available: bool,
    /// Что выгодно везти оттуда сюда.
    pub cheaper: Vec<String>,
    /// Что выгодно везти отсюда туда.
    pub dearer: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct JourneyView {
    pub from: String,
    pub to: String,
    pub arrives: String,
    pub days_left: u32,
}
