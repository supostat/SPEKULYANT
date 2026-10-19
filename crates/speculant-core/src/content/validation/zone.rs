//! Проверки контента зоны: правила, зоны эпох и услуги.

use std::collections::BTreeSet;

use super::super::loader::Parsed;
use super::super::{ContentError, Ending, ZoneService};
use super::{Known, Report, check_reference};
use crate::life::EndingKind;
use crate::zone::{Camp, ZoneRules};

pub(super) fn validate(parsed: &Parsed<'_>, known: &Known, errors: &mut Vec<ContentError>) {
    if let Some((file, rules)) = &parsed.zone {
        let mut report = Report {
            file,
            errors: &mut *errors,
        };
        let life_ending = parsed
            .endings
            .iter()
            .find(|entry| entry.id == rules.life_ending.as_str())
            .map(|entry| &entry.value);
        check_rules(rules, life_ending, known, &mut report);
        check_camps(rules, known, &mut report);
    }
    for entry in &parsed.zone_services {
        check_service(
            &entry.value,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
}

fn check_rules(
    rules: &ZoneRules,
    life_ending: Option<&Ending>,
    known: &Known,
    report: &mut Report<'_, '_>,
) {
    for (key, value) in [
        ("week_days", rules.week_days),
        ("longest_sentence_months", rules.longest_sentence_months),
        ("parole_retry_weeks", rules.parole_retry_weeks),
    ] {
        if value == 0 {
            report.at(report.file.line_of(key), format!("{key}: больше нуля"));
        }
    }
    match life_ending {
        None => report.at_reference(
            rules.life_ending.as_str(),
            format!("финала {} нет в endings/", rules.life_ending),
        ),
        Some(ending) if ending.kind != EndingKind::Prison => report.at_reference(
            rules.life_ending.as_str(),
            format!("финал срока {} — не зона", rules.life_ending),
        ),
        Some(_) => {}
    }
    for (key, chance) in [
        ("event_chance", rules.event_chance),
        ("health_loss_chance", rules.health_loss_chance),
        ("idle_authority_chance", rules.idle_authority_chance),
    ] {
        check_share(key, chance, report);
    }
    if rules.health_loss < 0 {
        report.at(
            report.file.line_of("health_loss ="),
            "health_loss: зона здоровья не прибавляет".to_owned(),
        );
    }
    let levels = rules.authority_levels.len();
    let levels_line = report.file.line_of("authority_levels");
    if levels == 0
        || rules
            .authority_levels
            .iter()
            .any(|name| name.trim().is_empty())
    {
        report.at(
            levels_line,
            "у авторитета нет уровней или у уровня нет слова".to_owned(),
        );
    }
    for (key, level) in [
        ("start_authority", rules.start_authority),
        ("parole_max_authority", rules.parole_max_authority),
    ] {
        if !(1..=levels).contains(&usize::from(level)) {
            report.at(
                report.file.line_of(key),
                format!("{key} = {level}: уровни авторитета — от 1 до {levels}"),
            );
        }
    }
    if rules.ties.len() != levels {
        report.at(
            report.file.line_of("ties ="),
            format!(
                "связей «оттуда» {} значений, а уровней авторитета {levels}",
                rules.ties.len()
            ),
        );
    }
    check_reference(
        &known.factions,
        rules.ties_faction.as_str(),
        "фракции",
        "factions/",
        report,
    );
    if !rules.parcel.price.is_positive() || rules.parcel.contents.is_empty() {
        report.at(
            report.file.line_of("parcel"),
            "передача стоит больше нуля и что-то приносит".to_owned(),
        );
    }
    let trustees = &rules.trustees;
    if let Some(scale) = known.trait_scale {
        for (key, chances) in [
            ("vanish_chances", &trustees.vanish_chances),
            ("steal_chances", &trustees.steal_chances),
        ] {
            if chances.len() != usize::from(scale) {
                report.at(
                    report.file.line_of(key),
                    format!("{key}: {} значений, а делений черт {scale}", chances.len()),
                );
            }
            for chance in chances {
                check_share(key, *chance, report);
            }
        }
    }
    if !(trustees.steal_share > 0.0 && trustees.steal_share <= 1.0) {
        report.at(
            report.file.line_of("steal_share"),
            format!(
                "steal_share = {}: ворующий уносит долю больше нуля и не больше всего",
                trustees.steal_share
            ),
        );
    }
}

/// Зона у каждой эпохи календаря и ни одной лишней.
fn check_camps(rules: &ZoneRules, known: &Known, report: &mut Report<'_, '_>) {
    let covered: BTreeSet<&str> = rules.camps.keys().map(|epoch| epoch.as_str()).collect();
    for epoch in &known.epochs {
        if !covered.contains(epoch.as_str()) {
            report.at(
                report.file.line_of("camps"),
                format!("нет зоны для эпохи {epoch}"),
            );
        }
    }
    for (epoch, camp) in &rules.camps {
        if !known.epochs.contains(epoch.as_str()) {
            report.at(
                report.file.line_of(&format!("camps.{epoch}")),
                format!("эпохи {epoch} нет в календаре epochs/time.toml"),
            );
        }
        check_camp(epoch.as_str(), camp, report);
    }
}

fn check_camp(epoch: &str, camp: &Camp, report: &mut Report<'_, '_>) {
    let line = report.file.line_of(&format!("camps.{epoch}"));
    if camp.title.trim().is_empty() {
        report.at(line, format!("у зоны эпохи {epoch} нет названия"));
    }
    if camp.parcel_weeks == 0 {
        report.at(
            line,
            format!("зона эпохи {epoch}: передача не чаще раза в неделю"),
        );
    }
    if !(camp.parole_after > 0.0 && camp.parole_after <= 1.0) {
        report.at(
            line,
            format!(
                "зона эпохи {epoch}: parole_after = {} — доля срока больше нуля и не больше всего",
                camp.parole_after
            ),
        );
    }
    if !(0.0..=1.0).contains(&camp.parole_chance) {
        report.at(
            line,
            format!(
                "зона эпохи {epoch}: parole_chance = {} — доля от 0 до 1",
                camp.parole_chance
            ),
        );
    }
    if camp.bribe.is_some_and(|bribe| !bribe.is_positive()) {
        report.at(
            line,
            format!("зона эпохи {epoch}: взятка больше нуля; не берут — уберите bribe"),
        );
    }
}

/// Услуга чего-то стоит и что-то даёт.
fn check_service(service: &ZoneService, report: &mut Report<'_, '_>) {
    if service.title.trim().is_empty() {
        report.at(
            report.file.line_of("title"),
            "у услуги нет названия".to_owned(),
        );
    }
    if service.price.is_empty() {
        report.at(
            report.file.line_of("price"),
            "услуга бесплатной не бывает: цена в сигаретах или чае".to_owned(),
        );
    }
    if service.health == 0 && service.authority == 0 {
        report.at(
            None,
            "услуга ничего не даёт: ни здоровья, ни авторитета".to_owned(),
        );
    }
}

fn check_share(key: &str, share: f64, report: &mut Report<'_, '_>) {
    if !(0.0..=1.0).contains(&share) {
        report.at(
            report.file.line_of(key),
            format!("{key} = {share}: доля от 0 до 1"),
        );
    }
}
