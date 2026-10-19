use serde::Deserialize;

use crate::content::EndingId;
use crate::time::GameDate;

/// Правила жизни из `data/life.toml` (решение 15.4): шкала здоровья и старение, потери денег под
/// матрасом, зачёт жилья, газетная страница итогов, фраза некролога о деле, длина таблиц рекордов
/// и имена персонажей.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LifeRules {
    pub health_max: i32,
    pub start_health: i32,
    /// Состояния здоровья снизу вверх: с какого значения какое слово видит игрок.
    pub health_states: Vec<HealthState>,
    /// С этого возраста в каждый день рождения здоровье убывает на `aging_per_year`.
    pub aging_from: u32,
    pub aging_per_year: i32,
    /// Финал, когда здоровье кончилось.
    pub health_ending: EndingId,
    /// Какая доля денег под матрасом пропадает при краже или пожаре.
    pub stash_loss_share: f64,
    /// За такую долю цены прежнее жильё идёт в зачёт нового.
    pub resale_share: f64,
    /// С этой даты капитал в итогах считают в долларах.
    pub capital_in_dollars_from: GameDate,
    /// Страниц в газете: номер страницы заметки выбирает сид.
    pub newspaper_pages: u32,
    /// Сколько жизней помнит каждая таблица рекордов.
    pub records_size: usize,
    /// Фраза некролога о первом деле с подстановками `{year}` и `{business}`.
    pub business_sentence: String,
    pub names: NamePools,
}

/// Состояние здоровья, которое видит игрок, — с какого значения шкалы.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthState {
    pub from: i32,
    pub name: String,
}

/// Из чего сид собирает имя персонажа: «Лыков Пётр Сергеевич».
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NamePools {
    pub surnames: Vec<String>,
    pub first_names: Vec<String>,
    pub patronymics: Vec<String>,
}

impl LifeRules {
    /// Слово для здоровья: последнее состояние, до которого шкала дотягивает; на нуле и ниже —
    /// самое низкое.
    pub fn health_state(&self, health: i32) -> &str {
        self.health_states
            .iter()
            .rev()
            .find(|state| health >= state.from)
            .or_else(|| self.health_states.first())
            .map(|state| state.name.as_str())
            .expect("состояния здоровья есть — это проверяет загрузка")
    }
}
