use crate::content::{CityId, GoodId, NpcId};

/// Сделка челнока на толкучке города: что и сколько он скупил или привёз. Лента игрока показывает
/// её, если игрок стоит на этой толкучке.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RivalDeal {
    pub rival: NpcId,
    pub city: CityId,
    pub good: GoodId,
    pub quantity: u32,
    pub side: DealSide,
}

/// Скупил ли челнок товар на толкучке или привёз его туда.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DealSide {
    Bought,
    Sold,
}
