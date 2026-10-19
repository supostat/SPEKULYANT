use std::collections::BTreeMap;

use speculant_core::content::{ContentError, ContentFile, Event, EventId, TimelineNodeId};

/// Где в `data/` лежит сущность и на какой строке её поле: ошибки строгой проверки и покрытия
/// печатаются тем же «файл:строка: сообщение», что и ошибки загрузки.
pub struct Locator<'a> {
    files: BTreeMap<&'a str, &'a ContentFile>,
}

impl<'a> Locator<'a> {
    pub fn new(files: &'a [ContentFile]) -> Self {
        Self {
            files: files
                .iter()
                .map(|file| (file.path.as_str(), file))
                .collect(),
        }
    }

    /// Ошибка в файле на первой строке, где встречается `needle`; без него — про файл целиком.
    pub fn error(&self, path: &str, needle: Option<&str>, message: String) -> ContentError {
        let line =
            needle.and_then(|needle| self.files.get(path).and_then(|file| file.line_of(needle)));
        ContentError::new(path, line, message)
    }
}

/// Файл события: `events/<эпоха>/<имя>.toml` или `zone/events/<имя>.toml`.
pub fn event_path(id: &EventId, event: &Event) -> String {
    match &event.epoch {
        Some(epoch) => format!("events/{epoch}/{id}.toml"),
        None => format!("zone/events/{id}.toml"),
    }
}

/// Файл исторического узла.
pub fn node_path(node: &TimelineNodeId) -> String {
    format!("history/{node}.toml")
}

/// Файл шаблона генератора песочницы.
pub fn template_path(template: &TimelineNodeId) -> String {
    format!("history/post2004/{template}.toml")
}

/// Ссылка в кавычках — так идентификатор стоит в TOML.
pub fn quoted(id: &str) -> String {
    format!("\"{id}\"")
}
