//! Проверки контента после разбора: ссылки, флаги, окна дат, кривые, выборы и подстановки.

use std::collections::BTreeSet;

use super::loader::{Entry, Parsed};
use super::{Choice, Conditions, ContentError, ContentFile, Event};
use crate::time::{GameDate, TimeRules};

const PLACEHOLDERS: [&str; 3] = ["npc", "price", "sum"];
const NPC_PLACEHOLDER: &str = "npc";
const FIRST_YEAR: i32 = 1984;

/// Множества идентификаторов, на которые можно ссылаться.
struct Known {
    epochs: BTreeSet<String>,
    goods: BTreeSet<String>,
    categories: BTreeSet<String>,
    cities: BTreeSet<String>,
    npcs: BTreeSet<String>,
    events: BTreeSet<String>,
    flags_set: BTreeSet<String>,
}

/// Отчёт об ошибках одного файла.
struct Report<'a, 'e> {
    file: &'a ContentFile,
    errors: &'e mut Vec<ContentError>,
}

impl Report<'_, '_> {
    fn at(&mut self, line: Option<usize>, message: String) {
        self.errors
            .push(ContentError::new(&self.file.path, line, message));
    }

    fn at_reference(&mut self, id: &str, message: String) {
        let line = self.file.line_of(&format!("\"{id}\""));
        self.at(line, message);
    }
}

pub(super) fn validate(parsed: &Parsed<'_>, time: &TimeRules, errors: &mut Vec<ContentError>) {
    let known = Known {
        epochs: time
            .epochs
            .epochs()
            .iter()
            .map(|epoch| epoch.id.clone())
            .collect(),
        goods: ids(&parsed.goods),
        categories: parsed
            .goods
            .iter()
            .map(|entry| entry.value.category.clone())
            .collect(),
        cities: ids(&parsed.cities),
        npcs: ids(&parsed.npcs),
        events: ids(&parsed.events),
        flags_set: parsed
            .events
            .iter()
            .flat_map(|entry| entry.value.choices.iter())
            .flat_map(|choice| {
                choice
                    .consequences
                    .set_flags
                    .iter()
                    .map(|flag| flag.as_str().to_owned())
            })
            .collect(),
    };
    for entry in &parsed.epochs {
        if !known.epochs.contains(&entry.id) {
            Report {
                file: entry.file,
                errors: &mut *errors,
            }
            .at(
                None,
                format!("эпохи {} нет в календаре epochs/time.toml", entry.id),
            );
        }
    }
    for entry in &parsed.goods {
        check_good(
            entry,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.cities {
        check_city(
            entry,
            &known,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    if let Some((file, routes)) = &parsed.routes {
        check_routes(
            routes,
            &known,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.biographies {
        let city = entry.value.start_city.as_str();
        if !known.cities.contains(city) {
            Report {
                file: entry.file,
                errors: &mut *errors,
            }
            .at_reference(city, format!("стартового города {city} нет в cities/"));
        }
    }
    for entry in &parsed.events {
        check_event(
            &entry.value,
            &known,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
}

fn ids<T>(entries: &[Entry<'_, T>]) -> BTreeSet<String> {
    entries.iter().map(|entry| entry.id.clone()).collect()
}

fn check_good(entry: &Entry<'_, super::Good>, report: &mut Report<'_, '_>) {
    let curve = &entry.value.curve;
    let curve_line = report.file.line_of("curve");
    if curve.is_empty() {
        report.at(
            curve_line,
            "у товара нет ни одной опорной точки цены".to_owned(),
        );
    }
    for point in curve {
        if point.year < FIRST_YEAR {
            report.at(
                curve_line,
                format!(
                    "опорная точка {} раньше начала игры в {FIRST_YEAR} году",
                    point.year
                ),
            );
        }
        if !point.price.is_positive() {
            report.at(
                curve_line,
                format!("цена в {} году должна быть больше нуля", point.year),
            );
        }
    }
    if curve.windows(2).any(|pair| pair[1].year <= pair[0].year) {
        report.at(
            curve_line,
            "годы опорных точек кривой должны строго возрастать".to_owned(),
        );
    }
}

fn check_city(entry: &Entry<'_, super::City>, known: &Known, report: &mut Report<'_, '_>) {
    for (category, multiplier) in &entry.value.multipliers {
        if !known.categories.contains(category) {
            report.at(
                report.file.line_of(category),
                format!("категории {category} нет ни у одного товара"),
            );
        }
        if !(multiplier.is_finite() && *multiplier > 0.0) {
            report.at(
                report.file.line_of(category),
                format!("множитель категории {category} должен быть больше нуля"),
            );
        }
    }
    for (key, date) in [
        ("opens", entry.value.opens),
        ("connections_only_until", entry.value.connections_only_until),
    ] {
        if date.is_some_and(|date| date < GameDate::START) {
            report.at(
                report.file.line_of(key),
                format!("{key} раньше начала игры 1984-01-01"),
            );
        }
    }
}

fn check_routes(routes: &[super::Route], known: &Known, report: &mut Report<'_, '_>) {
    let mut seen = BTreeSet::new();
    for (index, route) in routes.iter().enumerate() {
        let route_line = report.file.nth_line_of("[[routes]]", index);
        for city in [route.from.as_str(), route.to.as_str()] {
            if !known.cities.contains(city) {
                report.at_reference(city, format!("города {city} нет в cities/"));
            }
        }
        if route.from == route.to {
            report.at(
                route_line,
                format!("маршрут из {} ведёт в тот же город", route.from),
            );
        }
        if route.days == 0 {
            report.at(
                route_line,
                format!(
                    "маршрут {} → {} не может занимать ноль дней",
                    route.from, route.to
                ),
            );
        }
        if !route.fare.is_positive() {
            report.at(
                route_line,
                format!(
                    "стоимость маршрута {} → {} должна быть больше нуля",
                    route.from, route.to
                ),
            );
        }
        if !seen.insert((route.from.as_str(), route.to.as_str())) {
            report.at(
                route_line,
                format!("маршрут {} → {} описан дважды", route.from, route.to),
            );
        }
    }
}

fn check_event(event: &Event, known: &Known, report: &mut Report<'_, '_>) {
    if let Some(epoch) = &event.epoch
        && !known.epochs.contains(epoch.as_str())
    {
        report.at(
            None,
            format!("каталога events/{epoch}/ нет среди эпох календаря epochs/time.toml"),
        );
    }
    if event.weight == 0 {
        report.at(
            report.file.line_of("weight"),
            "вес события должен быть больше нуля".to_owned(),
        );
    }
    if event.choices.is_empty() {
        report.at(None, "у события нет ни одного выбора".to_owned());
    }
    check_conditions(&event.conditions, known, report);
    let has_npc = event.conditions.npc.is_some();
    check_placeholders(&event.text, has_npc, report);
    for (index, choice) in event.choices.iter().enumerate() {
        check_choice(choice, index, has_npc, known, report);
    }
}

fn check_choice(
    choice: &Choice,
    index: usize,
    has_npc: bool,
    known: &Known,
    report: &mut Report<'_, '_>,
) {
    let choice_line = report.file.nth_line_of("[[choices]]", index);
    let has_effect = !choice.consequences.is_empty()
        || choice.outcome.is_some()
        || !choice.follow_ups.is_empty();
    if !has_effect {
        report.at(
            choice_line,
            format!("выбор {} без последствий, исхода и продолжения", index + 1),
        );
    }
    check_conditions(&choice.conditions, known, report);
    check_placeholders(&choice.text, has_npc, report);
    if let Some(outcome) = &choice.outcome {
        check_placeholders(outcome, has_npc, report);
    }
    let consequences = &choice.consequences;
    for change in &consequences.goods {
        check_reference(
            &known.goods,
            change.good.as_str(),
            "товара",
            "goods/",
            report,
        );
    }
    for change in &consequences.contacts {
        check_reference(&known.npcs, change.npc.as_str(), "человека", "npc/", report);
    }
    for shock in &consequences.price_shocks {
        check_reference(
            &known.goods,
            shock.good.as_str(),
            "товара",
            "goods/",
            report,
        );
        if let Some(city) = &shock.city {
            check_reference(&known.cities, city.as_str(), "города", "cities/", report);
        }
        if !(shock.factor.is_finite() && shock.factor > 0.0) {
            report.at(
                report.file.line_of("factor"),
                format!("множитель шока цены {} должен быть больше нуля", shock.good),
            );
        }
    }
    for follow_up in &choice.follow_ups {
        check_reference(
            &known.events,
            follow_up.event.as_str(),
            "события-продолжения",
            "events/",
            report,
        );
        if !(follow_up.chance > 0.0 && follow_up.chance <= 1.0) {
            report.at(
                report.file.line_of("chance"),
                format!("вероятность продолжения {} вне (0, 1]", follow_up.event),
            );
        }
        if follow_up.after_days == 0 {
            report.at(
                report.file.line_of("after_days"),
                format!(
                    "продолжение {} не может прийти через ноль дней",
                    follow_up.event
                ),
            );
        }
    }
}

fn check_conditions(conditions: &Conditions, known: &Known, report: &mut Report<'_, '_>) {
    for epoch in &conditions.epochs {
        check_reference(
            &known.epochs,
            epoch.as_str(),
            "эпохи",
            "epochs/time.toml",
            report,
        );
    }
    for city in &conditions.cities {
        check_reference(&known.cities, city.as_str(), "города", "cities/", report);
    }
    for good in &conditions.goods {
        check_reference(&known.goods, good.good.as_str(), "товара", "goods/", report);
    }
    if let Some(npc) = &conditions.npc {
        check_reference(&known.npcs, npc.as_str(), "человека", "npc/", report);
    }
    for flag in conditions.flags_set.iter().chain(&conditions.flags_unset) {
        if !known.flags_set.contains(flag.as_str()) {
            report.at_reference(
                flag.as_str(),
                format!("флаг {flag} не ставит ни одно последствие"),
            );
        }
    }
    if let Some(window) = &conditions.window {
        let window_line = report.file.line_of("window");
        if window.from < GameDate::START {
            report.at(
                window_line,
                format!(
                    "окно дат начинается {} — раньше начала игры 1984-01-01",
                    window.from.to_iso()
                ),
            );
        }
        if window.until.is_some_and(|until| until < window.from) {
            report.at(
                window_line,
                "окно дат заканчивается раньше, чем начинается".to_owned(),
            );
        }
    }
    for reputation in &conditions.reputation {
        let reputation_line = report.file.line_of(&format!("\"{}\"", reputation.faction));
        match (reputation.at_least, reputation.at_most) {
            (None, None) => report.at(
                reputation_line,
                format!("условие репутации {} без границы", reputation.faction),
            ),
            (Some(low), Some(high)) if low > high => {
                report.at(
                    reputation_line,
                    format!(
                        "у условия репутации {} нижняя граница выше верхней",
                        reputation.faction
                    ),
                );
            }
            _ => {}
        }
    }
    if let Some(money) = &conditions.money
        && !money.at_least.is_positive()
    {
        report.at(
            report.file.line_of("money"),
            "условие денег должно требовать сумму больше нуля".to_owned(),
        );
    }
}

fn check_reference(
    known: &BTreeSet<String>,
    id: &str,
    what: &str,
    place: &str,
    report: &mut Report<'_, '_>,
) {
    if !known.contains(id) {
        report.at_reference(id, format!("{what} {id} нет в {place}"));
    }
}

/// Подстановки `{имя}` в тексте: только известные, а `{npc}` — лишь при событии с участником.
fn check_placeholders(text: &str, has_npc: bool, report: &mut Report<'_, '_>) {
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            report.at(
                report.file.line_of(&rest[start..]),
                "незакрытая подстановка «{»".to_owned(),
            );
            return;
        };
        let name = &after[..end];
        let line = report.file.line_of(&format!("{{{name}}}"));
        if !PLACEHOLDERS.contains(&name) {
            report.at(
                line,
                format!("подстановки {{{name}}} нет, есть {{npc}}, {{price}}, {{sum}}"),
            );
        } else if name == NPC_PLACEHOLDER && !has_npc {
            report.at(
                line,
                "подстановка {npc} в событии без условия npc".to_owned(),
            );
        }
        rest = &after[end + 1..];
    }
}
