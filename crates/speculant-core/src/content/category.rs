use serde::Deserialize;

/// Категория товаров (§5.2) из `data/categories/<категория>.toml`: по ней город задаёт множитель
/// цены, а экран поездки называет, что в городе дешевле и что дороже.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Category {
    /// Название строчными, как оно стоит в перечне: «видео и музыка».
    pub title: String,
}
