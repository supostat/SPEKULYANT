use crate::content::CityId;
use crate::time::GameDate;

/// Поездка в пути: откуда, куда, когда выехал и в какой день приезжает.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Journey {
    pub from: CityId,
    pub to: CityId,
    pub departed: GameDate,
    pub arrives: GameDate,
}
