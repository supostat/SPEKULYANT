use std::fmt;

/// Почему файл не сохранить или не прочесть.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// Имя слота — латиница в нижнем регистре, цифры и `_`, до 32 знаков.
    InvalidSlot(String),
    Missing,
    Io(String),
    Corrupt(String),
    OtherFormat,
    Newer {
        version: u32,
        supported: u32,
    },
    /// Файл жизни сделан с другими данными игры.
    OtherContent,
}

impl fmt::Display for SaveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidSlot(slot) => write!(
                formatter,
                "«{slot}» — не имя слота: латиница в нижнем регистре, цифры и _, до 32 знаков"
            ),
            Self::Missing => write!(formatter, "файла нет"),
            Self::Io(problem) => write!(formatter, "диск не дал прочесть или записать: {problem}"),
            Self::Corrupt(problem) => write!(formatter, "файл испорчен: {problem}"),
            Self::OtherFormat => write!(formatter, "это не тот файл"),
            Self::Newer { version, supported } => write!(
                formatter,
                "файл версии {version}, а игра читает до версии {supported}"
            ),
            Self::OtherContent => write!(formatter, "файл сделан с другими данными игры"),
        }
    }
}

impl std::error::Error for SaveError {}
