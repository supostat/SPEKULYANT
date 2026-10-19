use serde::{Deserialize, Serialize};

use super::{LevelView, SummaryLineView};

/// Дело игрока — строка списка дел и экран управляющего (§13.3, п. 4): точки, форма
/// собственности, управляющий, политики, прогноз на месяц и отчёт за прошлый месяц. Какое дело
/// открыто на экране — забота экрана, как выделенная строка рынка.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessView {
    /// Идентификатор для команд: «тип@город».
    pub business: String,
    /// «Ларьки у «Тушинской» · 3 точки».
    pub title: String,
    pub city: String,
    pub form: String,
    /// Без управляющего дело стоит.
    pub manager: Option<ManagerView>,
    pub policies: Vec<PolicyView>,
    /// Прогноза нет, пока нет управляющего.
    pub forecast: Option<ForecastView>,
    /// «закрыто до 12.10.1995», пока дело закрыто.
    pub closed_until: Option<String>,
    pub report: Option<ReportView>,
}

/// Управляющий: имя, с какого года, черты делениями и привычки.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerView {
    pub employee: String,
    pub name: String,
    /// «с 1994 года».
    pub since: String,
    pub traits: Vec<LevelView>,
    pub habits: Vec<String>,
}

/// Политика: подпись строки, варианты по порядку стрелок и выбранный — номер от нуля, как в
/// команде `set_policy`.
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

/// Отчёт управляющего за месяц: «август 1995» и строки «Выручка … 12 400 р.».
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReportView {
    pub month: String,
    pub lines: Vec<SummaryLineView>,
}

/// Человек, которого можно нанять управляющим: черты делениями и привычки — кому доверять, игрок
/// решает сам.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CandidateView {
    pub candidate: String,
    pub name: String,
    pub role: String,
    pub traits: Vec<LevelView>,
    pub habits: Vec<String>,
}

/// Дело, которое можно открыть в городе игрока сейчас: тип, форма собственности дня, цена
/// точки и хватает ли на неё денег и места.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusinessKindView {
    pub kind: String,
    pub title: String,
    pub form: String,
    pub cost: String,
    pub available: bool,
}
