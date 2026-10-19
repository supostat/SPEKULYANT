/// Как игрок вышел на волю (§12).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Release {
    Term,
    Parole,
    Amnesty,
    Bribe,
}

impl Release {
    /// Как выход пишется в ленте: «Вышел на волю по УДО».
    pub fn title(self) -> &'static str {
        match self {
            Self::Term => "по сроку",
            Self::Parole => "по УДО",
            Self::Amnesty => "по амнистии",
            Self::Bribe => "за взятку",
        }
    }
}
