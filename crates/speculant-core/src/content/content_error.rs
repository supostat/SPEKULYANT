use std::fmt;

/// Ошибка контента с местом: «файл:строка: сообщение»; строки нет, если ошибка про файл целиком.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ContentError {
    pub file: String,
    pub line: Option<usize>,
    pub message: String,
}

impl ContentError {
    pub(crate) fn new(file: &str, line: Option<usize>, message: impl Into<String>) -> Self {
        Self {
            file: file.to_owned(),
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for ContentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.line {
            Some(line) => write!(formatter, "{}:{line}: {}", self.file, self.message),
            None => write!(formatter, "{}: {}", self.file, self.message),
        }
    }
}
