use std::fs;
use std::path::Path;

use super::ContentError;

/// Файл данных: путь относительно корня `data/` через `/` и текст.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContentFile {
    pub path: String,
    pub text: String,
}

impl ContentFile {
    pub fn new(path: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            text: text.into(),
        }
    }

    /// Номер строки (с единицы), на которую приходится байтовое смещение.
    pub(crate) fn line_at_offset(&self, offset: usize) -> usize {
        self.text[..offset.min(self.text.len())]
            .matches('\n')
            .count()
            + 1
    }

    /// Первая строка, содержащая `needle`.
    pub fn line_of(&self, needle: &str) -> Option<usize> {
        self.nth_line_of(needle, 0)
    }

    /// Строка `index`-го (с нуля) вхождения `needle`.
    pub fn nth_line_of(&self, needle: &str, index: usize) -> Option<usize> {
        self.text
            .lines()
            .enumerate()
            .filter(|(_, line)| line.contains(needle))
            .nth(index)
            .map(|(number, _)| number + 1)
    }
}

/// Все файлы каталога рекурсивно, по алфавиту; скрытые файлы и каталоги пропускаются.
pub(super) fn read_tree(dir: &Path) -> Result<Vec<ContentFile>, Vec<ContentError>> {
    let mut files = Vec::new();
    let mut errors = Vec::new();
    collect(dir, "", &mut files, &mut errors);
    if errors.is_empty() {
        Ok(files)
    } else {
        Err(errors)
    }
}

fn collect(dir: &Path, prefix: &str, files: &mut Vec<ContentFile>, errors: &mut Vec<ContentError>) {
    let shown = if prefix.is_empty() {
        dir.display().to_string()
    } else {
        prefix.to_owned()
    };
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(error) => {
            return errors.push(ContentError::new(
                &shown,
                None,
                format!("каталог не читается: {error}"),
            ));
        }
    };
    let mut entries: Vec<_> = entries.filter_map(Result::ok).collect();
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with('.') {
            continue;
        }
        let relative = if prefix.is_empty() {
            name.clone()
        } else {
            format!("{prefix}/{name}")
        };
        let path = entry.path();
        if path.is_dir() {
            collect(&path, &relative, files, errors);
            continue;
        }
        match fs::read_to_string(&path) {
            Ok(text) => files.push(ContentFile::new(relative, text)),
            Err(error) => errors.push(ContentError::new(
                &relative,
                None,
                format!("файл не читается как UTF-8: {error}"),
            )),
        }
    }
}
