use serde::Deserialize;

/// Разброс масштаба узла истории: сид выбирает множитель силы между `from` и `to`; без поля
/// масштаб не плавает.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScaleRange {
    pub from: f64,
    pub to: f64,
}

impl Default for ScaleRange {
    fn default() -> Self {
        Self { from: 1.0, to: 1.0 }
    }
}
