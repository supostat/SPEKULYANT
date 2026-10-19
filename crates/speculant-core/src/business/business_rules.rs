use serde::Deserialize;

/// Правила дел из `data/business/rules.toml`: шкала риска прогноза, черты сотрудников, доли
/// недостачи и откупа, шансы происшествий в месяц и конкуренция с делами соперников.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BusinessRules {
    /// Уровни риска прогноза снизу вверх — имя и шанс беды в месяц.
    pub risk_levels: Vec<RiskLevel>,
    /// С этого уровня прогноз горит тревогой.
    pub alarm_level: u8,
    /// Уровни наезда и воровства, от которых считают дело, управляющий и политики.
    pub base_raid: i32,
    pub base_theft: i32,
    /// Черты сотрудника — деления от 1 до `trait_scale`; `neutral_trait` ничего не сдвигает.
    pub trait_scale: u8,
    pub neutral_trait: u8,
    /// Доля выручки за каждое деление хватки выше или ниже середины.
    pub skill_step: f64,
    /// Выручка месяца колеблется на эту долю в обе стороны.
    pub revenue_noise: f64,
    /// Недостача — эта доля выручки месяца.
    pub theft_share: f64,
    /// Откуп на проверке — эта доля выручки месяца.
    pub bribe_share: f64,
    /// На столько дней управляющий закрывает дело при проверке.
    pub closure_days: u32,
    /// Шансы в месяц: крыша поднимает ставку, пожар.
    pub roof_raise_chance: f64,
    pub fire_chance: f64,
    /// Шансы в месяц, что управляющий уйдёт или сдаст, — по делениям лояльности от 1.
    pub departure_chances: Vec<f64>,
    pub betrayal_chances: Vec<f64>,
    /// На столько меняется отношение уволенного.
    pub fired_relation: i32,
    /// Каждая точка соперника того же типа в городе делит выручку: множитель 1 / (1 + c·n).
    pub competition: f64,
    /// Соперник открывает точку, только если после неё у него останется столько цен точки.
    pub rival_reserve: f64,
}

/// Уровень риска прогноза: «умеренный» и шанс беды в месяц.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RiskLevel {
    pub name: String,
    pub monthly_chance: f64,
}

impl BusinessRules {
    /// Уровень риска по сумме сдвигов — в пределах шкалы.
    pub fn risk_level(&self, steps: i32) -> u8 {
        let highest = i32::try_from(self.risk_levels.len()).expect("уровней риска меньше 2^31") - 1;
        u8::try_from(steps.clamp(0, highest)).expect("уровней риска меньше 256")
    }

    pub fn risk(&self, level: u8) -> &RiskLevel {
        &self.risk_levels[usize::from(level)]
    }

    /// Черта в пределах шкалы.
    pub fn trait_level(&self, level: i32) -> u8 {
        u8::try_from(level.clamp(1, i32::from(self.trait_scale))).expect("шкала черт меньше 256")
    }

    /// Шанс в месяц по делению черты: лояльности, честности.
    pub fn by_trait(chances: &[f64], level: u8) -> f64 {
        chances[usize::from(level) - 1]
    }
}
