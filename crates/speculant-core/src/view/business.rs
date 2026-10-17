use serde::{Deserialize, Serialize};

use super::LevelView;

/// Строка списка дел.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessView {
    pub business: String,
    pub title: String,
    pub summary: String,
}

/// Экран управляющего: политики, черты, привычки и прогноз на месяц.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerView {
    pub business: String,
    pub business_title: String,
    pub manager_name: String,
    pub manager_since: String,
    pub policies: Vec<PolicyView>,
    pub traits: Vec<LevelView>,
    pub habits: Vec<String>,
    pub forecast: ForecastView,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PolicyView {
    pub policy: String,
    pub label: String,
    pub options: Vec<String>,
    pub selected: usize,
}

/// Прогноз на месяц: выручка и риски строками, флаги — для цвета тревоги.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForecastView {
    pub revenue: String,
    pub revenue_falls: bool,
    pub raid_risk: String,
    pub raid_risk_high: bool,
    pub theft_risk: String,
    pub theft_risk_high: bool,
}
