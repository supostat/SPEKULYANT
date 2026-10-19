//! Проверки мелодии заставки: доля и ноты длятся, и мелодия не пустая.

use super::super::ContentError;
use super::super::loader::Parsed;
use super::Report;

pub(super) fn validate(parsed: &Parsed<'_>, errors: &mut Vec<ContentError>) {
    let Some((file, melody)) = &parsed.boot_melody else {
        return;
    };
    let mut report = Report { file, errors };
    if melody.beat_ms == 0 {
        report.at(file.line_of("beat_ms"), "beat_ms: больше нуля".to_owned());
    }
    if melody.notes.is_empty() {
        report.at(
            file.line_of("notes"),
            "в мелодии нет ни одной ноты".to_owned(),
        );
    }
    for (index, note) in melody.notes.iter().enumerate() {
        if note.beats == 0 {
            report.at(
                file.nth_line_of("beats", index),
                format!("нота {}: beats больше нуля", index + 1),
            );
        }
    }
}
