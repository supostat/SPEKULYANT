use serde::Deserialize;

/// Привычка или слабость сотрудника (§7.2) из `data/business/habits/<привычка>.toml`: строка для
/// экрана управляющего, сдвиги прогноза, как у вариантов политик, и шанс в месяц потерять деление
/// хватки — так человек спивается.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Habit {
    pub text: String,
    #[serde(default)]
    pub revenue: i32,
    #[serde(default)]
    pub raid: i32,
    #[serde(default)]
    pub theft: i32,
    #[serde(default)]
    pub skill_loss_chance: f64,
}
