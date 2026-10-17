use serde::{Deserialize, Serialize};

/// Пейджер, позже мобила-кирпич: заголовок устройства и лента сообщений.
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
