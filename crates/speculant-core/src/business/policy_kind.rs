use serde::{Deserialize, Serialize};

/// Политика управляющего (§7.3). Набор политик и их роль в расчёте — правила ядра; подписи,
/// варианты и их влияние на прогноз — данные `data/business/policies/<политика>.toml`. Порядок
/// вариантов перечисления — порядок строк на экране управляющего.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PolicyKind {
    Markup,
    /// Варианты платят крыше долю выручки; крыша закрывает наезды рэкета.
    Roof,
    Purchase,
    NightTrade,
    /// Варианты говорят, что делает управляющий, когда приходит проверка.
    Inspection,
    /// Варианты платят персоналу долю зарплаты по рынку.
    Wage,
}

impl PolicyKind {
    pub const ALL: [Self; 6] = [
        Self::Markup,
        Self::Roof,
        Self::Purchase,
        Self::NightTrade,
        Self::Inspection,
        Self::Wage,
    ];

    /// Идентификатор так, как его пишут данные и команды: имя файла политики.
    pub fn id(self) -> &'static str {
        match self {
            Self::Markup => "markup",
            Self::Roof => "roof",
            Self::Purchase => "purchase",
            Self::NightTrade => "night_trade",
            Self::Inspection => "inspection",
            Self::Wage => "wage",
        }
    }

    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|kind| kind.id() == id)
    }
}
