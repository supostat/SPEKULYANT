//! Проверки контента жизни: правила здоровья и итогов, жильё, финалы и биографии.

use std::collections::BTreeSet;

use super::super::loader::Parsed;
use super::super::{Biography, ContentError, Ending, Housing};
use super::{Known, Report, check_reference};
use crate::life::{EndingKind, LifeRules};

/// Подстановки фраз финала.
const ENDING_PLACEHOLDERS: [&str; 6] = ["name", "NAME", "born", "ended", "date", "age"];
/// Подстановки фразы некролога о деле.
const BUSINESS_PLACEHOLDERS: [&str; 2] = ["year", "business"];

pub(super) fn validate(parsed: &Parsed<'_>, known: &Known, errors: &mut Vec<ContentError>) {
    if let Some((file, rules)) = &parsed.life {
        let endings: Vec<(&str, &Ending)> = parsed
            .endings
            .iter()
            .map(|entry| (entry.id.as_str(), &entry.value))
            .collect();
        check_rules(
            rules,
            &endings,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.housing {
        check_housing(
            &entry.value,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.endings {
        check_ending(
            &entry.value,
            known,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    let homes: Vec<_> = parsed
        .money_places
        .iter()
        .filter(|entry| entry.value.home)
        .collect();
    for extra in homes.iter().skip(1) {
        Report {
            file: extra.file,
            errors: &mut *errors,
        }
        .at(
            extra.file.line_of("home"),
            format!(
                "дома деньги лежат в одном месте, а оно уже есть: {}",
                homes[0].id
            ),
        );
    }
}

/// Биография: возраст на старте, известное жильё, первая фраза некролога и открытие — флаг с
/// подсказкой у закрытой и ни того ни другого у открытой.
pub(super) fn check_biography(biography: &Biography, known: &Known, report: &mut Report<'_, '_>) {
    if biography.age.from > biography.age.to {
        report.at(
            report.file.line_of("age"),
            "возраст на старте: from больше to".to_owned(),
        );
    }
    check_reference(
        &known.housing,
        biography.housing.as_str(),
        "жилья",
        "housing/",
        report,
    );
    if biography.obituary.trim().is_empty() {
        report.at(
            report.file.line_of("obituary"),
            "у биографии нет первой фразы некролога".to_owned(),
        );
    }
    let locked = !biography.starts_unlocked;
    match (&biography.unlocked_by_flag, &biography.unlock_hint) {
        (Some(flag), Some(_)) if locked => {
            if !known.flags_set.contains(flag.as_str()) {
                report.at_reference(
                    flag.as_str(),
                    format!("флаг {flag} не ставит ни одно последствие — биографию не открыть"),
                );
            }
        }
        (None, None) if !locked => {}
        _ if locked => report.at(
            report.file.line_of("starts_unlocked"),
            "закрытой биографии нужны флаг открытия unlocked_by_flag и подсказка unlock_hint"
                .to_owned(),
        ),
        _ => report.at(
            report.file.line_of("starts_unlocked"),
            "открытой с начала биографии не нужны unlocked_by_flag и unlock_hint".to_owned(),
        ),
    }
}

fn check_rules(rules: &LifeRules, endings: &[(&str, &Ending)], report: &mut Report<'_, '_>) {
    if rules.health_max <= 0 || !(1..=rules.health_max).contains(&rules.start_health) {
        report.at(
            report.file.line_of("start_health"),
            format!(
                "здоровье на старте {} вне шкалы 1–{}",
                rules.start_health, rules.health_max
            ),
        );
    }
    let states_line = report.file.line_of("health_states");
    match rules.health_states.first() {
        None => report.at(states_line, "у здоровья нет ни одного состояния".to_owned()),
        Some(lowest) if lowest.from != 1 => report.at(
            states_line,
            format!(
                "первое состояние здоровья начинается с {}, а шкала — с 1",
                lowest.from
            ),
        ),
        Some(_) => {}
    }
    if rules
        .health_states
        .windows(2)
        .any(|pair| pair[1].from <= pair[0].from)
    {
        report.at(
            states_line,
            "пороги состояний здоровья должны строго расти".to_owned(),
        );
    }
    if rules
        .health_states
        .iter()
        .any(|state| state.from > rules.health_max || state.name.trim().is_empty())
    {
        report.at(
            states_line,
            "состояние здоровья без имени или выше шкалы".to_owned(),
        );
    }
    if rules.aging_per_year < 0 {
        report.at(
            report.file.line_of("aging_per_year"),
            "годы здоровья не прибавляют: aging_per_year не меньше нуля".to_owned(),
        );
    }
    match endings
        .iter()
        .find(|(id, _)| *id == rules.health_ending.as_str())
    {
        None => report.at_reference(
            rules.health_ending.as_str(),
            format!("финала {} нет в endings/", rules.health_ending),
        ),
        Some((_, ending)) if ending.kind != EndingKind::Death => report.at_reference(
            rules.health_ending.as_str(),
            format!("финал здоровья {} — не смерть", rules.health_ending),
        ),
        Some(_) => {}
    }
    for (key, share) in [
        ("stash_loss_share", rules.stash_loss_share),
        ("resale_share", rules.resale_share),
    ] {
        if !(0.0..=1.0).contains(&share) {
            report.at(
                report.file.line_of(key),
                format!("{key} = {share}: доля от 0 до 1"),
            );
        }
    }
    if rules.newspaper_pages == 0 {
        report.at(
            report.file.line_of("newspaper_pages"),
            "в газете хотя бы одна страница".to_owned(),
        );
    }
    check_placeholders(
        &rules.business_sentence,
        &BUSINESS_PLACEHOLDERS,
        "business_sentence",
        report,
    );
    let pools = [
        ("surnames", &rules.names.surnames),
        ("first_names", &rules.names.first_names),
        ("patronymics", &rules.names.patronymics),
    ];
    for (key, pool) in pools {
        if pool.is_empty() {
            report.at(
                report.file.line_of(key),
                format!("{key}: не из чего собрать имя"),
            );
        }
    }
}

fn check_housing(housing: &Housing, report: &mut Report<'_, '_>) {
    if housing.price.is_some_and(|price| !price.is_positive()) {
        report.at(
            report.file.line_of("price"),
            "цена жилья больше нуля; жильё без цены не продаётся".to_owned(),
        );
    }
    if housing.on_sale_from.is_some() && housing.price.is_none() {
        report.at(
            report.file.line_of("on_sale_from"),
            "жильё без цены не продаётся: on_sale_from без price".to_owned(),
        );
    }
    if !(0.0..=1.0).contains(&housing.stash_risk) {
        report.at(
            report.file.line_of("stash_risk"),
            format!("stash_risk = {}: доля от 0 до 1", housing.stash_risk),
        );
    }
}

/// Финал: заметка для каждой эпохи календаря, добровольным бывает только уход, окно — только у
/// добровольного, и подстановки — из известных.
fn check_ending(ending: &Ending, known: &Known, report: &mut Report<'_, '_>) {
    let covered: BTreeSet<&str> = ending.sources.keys().map(|epoch| epoch.as_str()).collect();
    for epoch in &known.epochs {
        if !covered.contains(epoch.as_str()) {
            report.at(
                report.file.line_of("sources"),
                format!("нет заметки для эпохи {epoch}"),
            );
        }
    }
    for epoch in &covered {
        if !known.epochs.contains(*epoch) {
            report.at(
                report.file.line_of(epoch),
                format!("эпохи {epoch} нет в календаре epochs/time.toml"),
            );
        }
    }
    if ending.voluntary && ending.kind != EndingKind::Departure {
        report.at(
            report.file.line_of("voluntary"),
            "по своей воле жизнь кончается только уходом: voluntary — у kind = \"departure\""
                .to_owned(),
        );
    }
    if ending.available_from.is_some() && !ending.voluntary {
        report.at(
            report.file.line_of("available_from"),
            "окно available_from бывает только у добровольного финала".to_owned(),
        );
    }
    if ending.capital_label.trim().is_empty() {
        report.at(
            report.file.line_of("capital_label"),
            "у финала нет подписи капитала".to_owned(),
        );
    }
    check_placeholders(&ending.headline, &ENDING_PLACEHOLDERS, "headline", report);
    check_placeholders(&ending.text, &ENDING_PLACEHOLDERS, "text", report);
}

fn check_placeholders(text: &str, allowed: &[&str], key: &str, report: &mut Report<'_, '_>) {
    let mut rest = text;
    while let Some(start) = rest.find('{') {
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            report.at(
                report.file.line_of(key),
                format!("{key}: незакрытая подстановка «{{»"),
            );
            return;
        };
        let name = &after[..end];
        if !allowed.contains(&name) {
            report.at(
                report.file.line_of(key),
                format!(
                    "{key}: подстановки {{{name}}} нет, есть {}",
                    allowed
                        .iter()
                        .map(|allowed| format!("{{{allowed}}}"))
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            );
        }
        rest = &after[end + 1..];
    }
}
