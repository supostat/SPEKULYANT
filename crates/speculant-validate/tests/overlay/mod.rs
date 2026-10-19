//! Фикстура валидатора — минимальный набор ядра, поверх которого кладутся файлы надстроек из
//! `tests/fixtures/<надстройка>/`: файл с тем же путём заменяется, новый добавляется.

use std::path::Path;

use speculant_core::Content;
use speculant_core::content::ContentFile;

const CONTENT_MIN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../speculant-core/tests/fixtures/content-min"
);
const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

/// Минимальный набор с надстройками по порядку; набор проходит загрузку — дефект надстройки виден
/// только строгой проверке или покрытию.
pub fn load(overlays: &[&str]) -> (Content, Vec<ContentFile>) {
    let mut files =
        Content::read_files(Path::new(CONTENT_MIN)).expect("минимальный набор читается");
    for overlay in overlays {
        let added = Content::read_files(&Path::new(FIXTURES).join(overlay))
            .unwrap_or_else(|errors| panic!("надстройка {overlay} не читается: {errors:?}"));
        for file in added {
            match files.iter_mut().find(|existing| existing.path == file.path) {
                Some(existing) => *existing = file,
                None => files.push(file),
            }
        }
    }
    let content = Content::from_files(files.clone()).unwrap_or_else(|errors| {
        let shown: Vec<String> = errors.iter().map(ToString::to_string).collect();
        panic!("набор {overlays:?} не загружается:\n{}", shown.join("\n"))
    });
    (content, files)
}
