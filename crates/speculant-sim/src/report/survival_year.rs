use serde::Serialize;

use crate::LifeOutcome;

/// Точка кривой выживания: какая доля жизней к концу года не кончилась бедой — смертью, сроком
/// или разорением; уход по своей воле бедой не считается.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct SurvivalYear {
    pub year: i32,
    pub alive: f64,
}

impl SurvivalYear {
    /// Кривая по годам `years`; `lost_on` — год беды каждой жизни, если она была.
    pub(crate) fn curve(
        years: impl Iterator<Item = i32>,
        outcomes: &[LifeOutcome],
        lost_on: impl Fn(&LifeOutcome) -> Option<i32>,
    ) -> Vec<Self> {
        let losses: Vec<i32> = outcomes.iter().filter_map(lost_on).collect();
        years
            .map(|year| {
                let lost = losses.iter().filter(|lost| **lost <= year).count();
                Self {
                    year,
                    alive: 1.0 - lost as f64 / outcomes.len() as f64,
                }
            })
            .collect()
    }
}
