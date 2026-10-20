use serde::Deserialize;

/// Судьба соперника без подробностей — для условий событий (§8, §10.1): в деле, в зоне,
/// разорился, погиб или депутат.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FateKind {
    #[default]
    Active,
    Prison,
    Ruined,
    Dead,
    Deputy,
}
