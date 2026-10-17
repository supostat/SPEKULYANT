use serde::{Deserialize, Serialize};

/// Пейджер, позже мобила-кирпич: заголовок устройства и лента сообщений. В снимке его нет, пока
/// устройства у игрока нет.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PagerView {
    pub device: String,
    pub messages: Vec<PagerMessageView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PagerMessageView {
    pub time: String,
    pub text: String,
    pub urgent: bool,
}
