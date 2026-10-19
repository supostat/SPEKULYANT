use std::fmt;

use serde::{Deserialize, Serialize};

macro_rules! content_id {
    ($(#[$documentation:meta])* $name:ident) => {
        $(#[$documentation])*
        #[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(id: impl Into<String>) -> Self {
                Self(id.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

content_id!(
    /// Товар — имя файла в `data/goods/`.
    GoodId
);
content_id!(
    /// Город — имя файла в `data/cities/`.
    CityId
);
content_id!(
    /// Эпоха из календаря `data/epochs/time.toml`.
    EpochId
);
content_id!(
    /// Стартовая биография — имя файла в `data/biographies/`.
    BiographyId
);
content_id!(
    /// Место хранения денег — имя файла в `data/money/`.
    MoneyPlaceId
);
content_id!(
    /// Шаблон соперника или знакомого — имя файла в `data/npc/`.
    NpcId
);
content_id!(
    /// Событие — имя файла в `data/events/<эпоха>/` или `data/zone/events/`.
    EventId
);
content_id!(
    /// Флаг прошлого решения: ставится последствием события, читается условием другого.
    FlagId
);
content_id!(
    /// Фракция репутации; набор фракций — решение 15.6.
    FactionId
);
content_id!(
    /// Узел таймлайна (§3.6) — имя файла в `data/history/` или шаблона в `data/history/post2004/`.
    TimelineNodeId
);
content_id!(
    /// Тип дела — имя файла в `data/business/kinds/`.
    BusinessKindId
);
content_id!(
    /// Форма собственности — имя файла в `data/business/forms/`.
    BusinessFormId
);
content_id!(
    /// Привычка сотрудника — имя файла в `data/business/habits/`.
    HabitId
);
content_id!(
    /// Жильё — имя файла в `data/housing/`.
    HousingId
);
content_id!(
    /// Финал жизни — имя файла в `data/endings/`.
    EndingId
);
content_id!(
    /// Услуга зоны — имя файла в `data/zone/services/`.
    ZoneServiceId
);
