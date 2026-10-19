use serde::Deserialize;

use crate::business::InspectionResponse;

/// Политика управляющего (§7.3) из `data/business/policies/<политика>.toml`: подпись строки,
/// вариант новых дел и варианты по порядку стрелок ◄ ►.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub title: String,
    /// Подпись варианта, с которого начинает новое дело.
    pub default: String,
    pub options: Vec<PolicyOption>,
}

/// Вариант политики: подпись и сдвиги прогноза — выручка в процентах, уровни наезда и воровства,
/// деления лояльности управляющего. Поля крыши, проверки и зарплаты есть только у своих политик.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PolicyOption {
    pub label: String,
    #[serde(default)]
    pub revenue: i32,
    #[serde(default)]
    pub raid: i32,
    #[serde(default)]
    pub theft: i32,
    #[serde(default)]
    pub loyalty: i32,
    /// Крыше — столько процентов выручки; ноль — крыши нет.
    pub roof_percent: Option<u32>,
    pub inspection: Option<InspectionResponse>,
    /// Зарплата — столько процентов от рынка.
    pub wage_percent: Option<u32>,
}

impl Policy {
    /// Номер варианта по подписи.
    pub fn option_index(&self, label: &str) -> Option<usize> {
        self.options.iter().position(|option| option.label == label)
    }
}
