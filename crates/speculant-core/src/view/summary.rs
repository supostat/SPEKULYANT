use serde::{Deserialize, Serialize};

/// Итоги жизни: газетная шапка, некролог или приговор, сводка и открытия для следующей жизни.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifeSummaryView {
    pub source: String,
    pub headline: String,
    pub text: String,
    pub lines: Vec<SummaryLineView>,
    pub discoveries: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SummaryLineView {
    pub label: String,
    pub value: String,
}
