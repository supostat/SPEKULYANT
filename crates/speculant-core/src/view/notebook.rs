use serde::{Deserialize, Serialize};

/// Запись записной книжки — знание, переходящее из жизни в жизнь.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NotebookEntryView {
    pub topic: String,
    pub title: String,
    pub text: String,
}
