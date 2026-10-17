use serde::{Deserialize, Serialize};

/// Событие, остановившее время: текст, выборы 1–N и, после выбора, исход.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct EventView {
    pub title: String,
    pub text: String,
    pub choices: Vec<ChoiceView>,
    pub outcome: Option<OutcomeView>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChoiceView {
    pub text: String,
    pub available: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutcomeView {
    pub text: String,
    pub effects: String,
}
