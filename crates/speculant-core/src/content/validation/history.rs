//! Проверки истории: правила генератора песочницы, исторические узлы, шаблоны и их исходы.

use std::collections::BTreeSet;

use super::super::loader::Parsed;
use super::super::{ContentError, HistoryNode, HistoryOutcome, HistoryTemplate, ScaleRange};
use super::{Known, Report, check_price_shock, check_reference};
use crate::history::HistoryGenerator;
use crate::time::{GameDate, TimeRules};

pub(super) fn validate(
    parsed: &Parsed<'_>,
    known: &Known,
    time: &TimeRules,
    errors: &mut Vec<ContentError>,
) {
    let generator = parsed.history_generator.as_ref().map(|(file, generator)| {
        check_generator(
            generator,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
        generator
    });
    let unfit_events = unfit_events(parsed);
    let templates: BTreeSet<&str> = parsed
        .history_templates
        .iter()
        .map(|entry| entry.id.as_str())
        .collect();
    for entry in &parsed.history {
        let mut report = Report {
            file: entry.file,
            errors: &mut *errors,
        };
        if templates.contains(entry.id.as_str()) {
            report.at(
                None,
                format!(
                    "узел {} совпадает по имени с шаблоном history/post2004/ — условие timeline их не различит",
                    entry.id
                ),
            );
        }
        check_node(&entry.value, generator, time, &mut report);
        check_outcomes(&entry.value.outcomes, known, &unfit_events, &mut report);
    }
    for entry in &parsed.history_templates {
        let mut report = Report {
            file: entry.file,
            errors: &mut *errors,
        };
        check_template(&entry.value, &mut report);
        check_outcomes(&entry.value.outcomes, known, &unfit_events, &mut report);
    }
}

/// События, которые узел не может начать: дорожные и происшествия с делами приходят своим путём.
fn unfit_events(parsed: &Parsed<'_>) -> BTreeSet<String> {
    parsed
        .events
        .iter()
        .filter(|entry| entry.value.road || entry.value.incident.is_some())
        .map(|entry| entry.id.clone())
        .collect()
}

fn check_generator(generator: &HistoryGenerator, report: &mut Report<'_, '_>) {
    if generator.from < GameDate::START || generator.until <= generator.from {
        report.at(
            report.file.line_of("until"),
            "генератор работает от from до until, и from — не раньше 1984-01-01".to_owned(),
        );
    }
    let gaps = generator.gap_days;
    if gaps.from == 0 || gaps.from > gaps.to {
        report.at(
            report.file.line_of("gap_days"),
            "промежуток между узлами — от дня, и from не больше to".to_owned(),
        );
    }
}

/// Окно узла: в игре, не пустое, в одной эпохе — рамки эпох не плавают — и до начала генератора.
fn check_node(
    node: &HistoryNode,
    generator: Option<&HistoryGenerator>,
    time: &TimeRules,
    report: &mut Report<'_, '_>,
) {
    check_title(&node.title, report);
    let window = node.window;
    let window_line = report.file.line_of("window");
    if window.from < GameDate::START || window.until <= window.from {
        report.at(
            window_line,
            "окно узла — не раньше 1984-01-01, и until позже from".to_owned(),
        );
        return;
    }
    let first = &time.epochs.epoch_on(window.from).id;
    let last = &time.epochs.epoch_on(window.until.previous()).id;
    if first != last {
        report.at(
            window_line,
            format!("окно узла захватывает две эпохи — {first} и {last}: рамки эпох не плавают"),
        );
    }
    if let Some(generator) = generator
        && window.until > generator.from
    {
        report.at(
            window_line,
            format!(
                "окно узла заходит за начало генератора песочницы {}",
                generator.from.to_iso()
            ),
        );
    }
    check_scale(node.scale, report);
}

fn check_template(template: &HistoryTemplate, report: &mut Report<'_, '_>) {
    check_title(&template.title, report);
    if template.weight == 0 {
        report.at(
            report.file.line_of("weight"),
            "вес шаблона должен быть больше нуля".to_owned(),
        );
    }
    check_scale(template.scale, report);
}

fn check_title(title: &str, report: &mut Report<'_, '_>) {
    if title.trim().is_empty() {
        report.at(
            report.file.line_of("title"),
            "у узла нет названия".to_owned(),
        );
    }
}

fn check_scale(scale: ScaleRange, report: &mut Report<'_, '_>) {
    let positive = scale.from.is_finite() && scale.to.is_finite() && scale.from > 0.0;
    if !positive || scale.from > scale.to {
        report.at(
            report.file.line_of("scale"),
            format!(
                "масштаб от {} до {}: больше нуля, и from не больше to",
                scale.from, scale.to
            ),
        );
    }
}

/// Исходы: хотя бы один, у каждого вес и новость, шоки — как у событий, событие — из данных и
/// не дорожное и не происшествие.
fn check_outcomes(
    outcomes: &[HistoryOutcome],
    known: &Known,
    unfit_events: &BTreeSet<String>,
    report: &mut Report<'_, '_>,
) {
    if outcomes.is_empty() {
        report.at(None, "у узла нет ни одного исхода".to_owned());
    }
    for (index, outcome) in outcomes.iter().enumerate() {
        let line = report.file.nth_line_of("[[outcomes]]", index);
        let number = index + 1;
        if outcome.weight == 0 {
            report.at(line, format!("исход {number}: вес больше нуля"));
        }
        if outcome.news.trim().is_empty() {
            report.at(line, format!("исход {number}: нет новости для ленты"));
        }
        for shock in &outcome.price_shocks {
            check_price_shock(shock, known, report);
        }
        if let Some(event) = &outcome.event {
            check_reference(&known.events, event.as_str(), "события", "events/", report);
            if unfit_events.contains(event.as_str()) {
                report.at_reference(
                    event.as_str(),
                    format!(
                        "исход {number}: событие {event} дорожное или происшествие — узел его не начнёт"
                    ),
                );
            }
        }
    }
}
