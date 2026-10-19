use serde::Deserialize;

use crate::content::GoodId;

/// Пейджер, позже мобила (§13.2): какие товары кармана служат устройством — первое по списку,
/// что есть у игрока, — и сколько последних сообщений держит лента.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagerRules {
    pub messages: usize,
    pub devices: Vec<PagerDevice>,
}

/// Устройство: товар кармана и заголовок экрана с подстановкой `{number}` — номером абонента.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PagerDevice {
    pub good: GoodId,
    pub title: String,
}
