//! Проверки контента дел: правила, формы собственности, политики, привычки, типы дел и черты
//! людей, которых можно нанять.

use std::collections::{BTreeMap, BTreeSet};

use super::super::loader::{Entry, Parsed};
use super::super::{BusinessForm, BusinessKind, ContentError, EmployeeTraits, Habit, Policy};
use super::{Known, Report, check_reference};
use crate::business::{BusinessRules, PolicyKind};
use crate::time::GameDate;

pub(super) fn validate(parsed: &Parsed<'_>, known: &Known, errors: &mut Vec<ContentError>) {
    if let Some((file, rules)) = &parsed.business_rules {
        check_rules(
            rules,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.business_forms {
        check_form(
            &entry.value,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.policies {
        check_policy(
            entry,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.habits {
        check_habit(
            &entry.value,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    let forms: BTreeMap<&str, &BusinessForm> = parsed
        .business_forms
        .iter()
        .map(|entry| (entry.id.as_str(), &entry.value))
        .collect();
    for entry in &parsed.business_kinds {
        check_kind(
            &entry.value,
            &forms,
            known,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
}

/// Черты человека, которого можно нанять: деления в пределах шкалы правил и известные привычки.
pub(super) fn check_employee(traits: &EmployeeTraits, known: &Known, report: &mut Report<'_, '_>) {
    if let Some(scale) = known.trait_scale {
        for (key, level) in [
            ("skill", traits.skill),
            ("honesty", traits.honesty),
            ("loyalty", traits.loyalty),
        ] {
            if !(1..=scale).contains(&level) {
                report.at(
                    report.file.line_of(key),
                    format!("{key} = {level}: черта от 1 до {scale} по шкале business/rules.toml"),
                );
            }
        }
    }
    for habit in &traits.habits {
        check_reference(
            &known.habits,
            habit.as_str(),
            "привычки",
            "business/habits/",
            report,
        );
    }
    if traits.from.is_some_and(|from| from < GameDate::START) {
        report.at(
            report.file.line_of("from"),
            "from раньше начала игры 1984-01-01".to_owned(),
        );
    }
}

fn check_rules(rules: &BusinessRules, report: &mut Report<'_, '_>) {
    if rules.risk_levels.is_empty() {
        report.at(
            report.file.line_of("risk_levels"),
            "у прогноза нет ни одного уровня риска".to_owned(),
        );
    }
    for level in &rules.risk_levels {
        if level.name.trim().is_empty() {
            report.at(
                report.file.line_of("risk_levels"),
                "у уровня риска нет имени".to_owned(),
            );
        }
        check_chance(
            &format!("риск «{}»", level.name),
            level.monthly_chance,
            report.file.line_of(&format!("\"{}\"", level.name)),
            report,
        );
    }
    if usize::from(rules.alarm_level) >= rules.risk_levels.len() {
        report.at(
            report.file.line_of("alarm_level"),
            format!(
                "тревога с уровня {}, а уровней всего {}",
                rules.alarm_level,
                rules.risk_levels.len()
            ),
        );
    }
    if rules.trait_scale == 0 || !(1..=rules.trait_scale).contains(&rules.neutral_trait) {
        report.at(
            report.file.line_of("neutral_trait"),
            format!(
                "середина черт {} вне шкалы 1–{}",
                rules.neutral_trait, rules.trait_scale
            ),
        );
    }
    for (key, chances) in [
        ("departure_chances", &rules.departure_chances),
        ("betrayal_chances", &rules.betrayal_chances),
    ] {
        let line = report.file.line_of(key);
        if chances.len() != usize::from(rules.trait_scale) {
            report.at(
                line,
                format!(
                    "{key}: шансов {}, а делений лояльности {}",
                    chances.len(),
                    rules.trait_scale
                ),
            );
        }
        for chance in chances {
            check_chance(key, *chance, line, report);
        }
    }
    for (key, share) in [
        ("roof_raise_chance", rules.roof_raise_chance),
        ("fire_chance", rules.fire_chance),
        ("theft_share", rules.theft_share),
        ("bribe_share", rules.bribe_share),
        ("revenue_noise", rules.revenue_noise),
    ] {
        check_chance(key, share, report.file.line_of(key), report);
    }
    for (key, value) in [
        ("skill_step", rules.skill_step),
        ("competition", rules.competition),
        ("rival_reserve", rules.rival_reserve),
    ] {
        if !(value.is_finite() && value >= 0.0) {
            report.at(
                report.file.line_of(key),
                format!("{key} = {value}: не меньше нуля"),
            );
        }
    }
    if rules.closure_days == 0 {
        report.at(
            report.file.line_of("closure_days"),
            "закрыться на ноль дней нельзя".to_owned(),
        );
    }
}

fn check_form(form: &BusinessForm, report: &mut Report<'_, '_>) {
    check_chance(
        "racket_share",
        form.racket_share,
        report.file.line_of("racket_share"),
        report,
    );
    check_chance("tax", form.tax, report.file.line_of("tax"), report);
    if form.until.is_some_and(|until| until <= form.from) {
        report.at(
            report.file.line_of("until"),
            "форма закрывается раньше, чем открывается".to_owned(),
        );
    }
}

/// Политика: файл назван по одной из шести политик ядра, варианты с разными подписями, вариант по
/// умолчанию среди них, поля крыши, проверки и зарплаты — у каждого варианта своей политики и
/// ни у какой другой.
fn check_policy(entry: &Entry<'_, Policy>, report: &mut Report<'_, '_>) {
    let Some(kind) = PolicyKind::from_id(&entry.id) else {
        let known: Vec<&str> = PolicyKind::ALL.iter().map(|kind| kind.id()).collect();
        report.at(
            None,
            format!(
                "политики {} нет: политики ядра — {}",
                entry.id,
                known.join(", ")
            ),
        );
        return;
    };
    let policy = &entry.value;
    if policy.options.len() < 2 {
        report.at(
            report.file.line_of("options"),
            "у политики меньше двух вариантов — выбирать не из чего".to_owned(),
        );
    }
    let mut labels = BTreeSet::new();
    for option in &policy.options {
        if !labels.insert(option.label.as_str()) {
            report.at(
                report.file.line_of(&format!("\"{}\"", option.label)),
                format!("вариант «{}» описан дважды", option.label),
            );
        }
    }
    if policy.option_index(&policy.default).is_none() {
        report.at(
            report.file.line_of("default"),
            format!(
                "варианта по умолчанию «{}» нет среди вариантов",
                policy.default
            ),
        );
    }
    for (index, option) in policy.options.iter().enumerate() {
        let line = report.file.nth_line_of("[[options]]", index);
        let fields = [
            (
                "roof_percent",
                PolicyKind::Roof,
                option.roof_percent.is_some(),
            ),
            (
                "inspection",
                PolicyKind::Inspection,
                option.inspection.is_some(),
            ),
            (
                "wage_percent",
                PolicyKind::Wage,
                option.wage_percent.is_some(),
            ),
        ];
        for (key, owner, present) in fields {
            if owner == kind && !present {
                report.at(
                    line,
                    format!("у варианта «{}» нет поля {key}", option.label),
                );
            }
            if owner != kind && present {
                report.at(
                    line,
                    format!(
                        "поле {key} бывает только у политики {}: вариант «{}»",
                        owner.id(),
                        option.label
                    ),
                );
            }
        }
        if option.wage_percent == Some(0) {
            report.at(
                line,
                format!("вариант «{}»: зарплата не может быть нулевой", option.label),
            );
        }
    }
}

fn check_habit(habit: &Habit, report: &mut Report<'_, '_>) {
    if habit.text.trim().is_empty() {
        report.at(
            report.file.line_of("text"),
            "у привычки нет строки для экрана".to_owned(),
        );
    }
    check_chance(
        "skill_loss_chance",
        habit.skill_loss_chance,
        report.file.line_of("skill_loss_chance"),
        report,
    );
}

/// Тип дела: известные формы без разрывов с первой открытой, цена точки и потолок точек, товар из
/// каталога, доля продаж и оборот, которые что-то приносят, и описанные политики.
fn check_kind(
    kind: &BusinessKind,
    forms: &BTreeMap<&str, &BusinessForm>,
    known: &Known,
    report: &mut Report<'_, '_>,
) {
    if kind.forms.is_empty() {
        report.at(
            report.file.line_of("forms"),
            "у дела нет ни одной формы собственности".to_owned(),
        );
    }
    for form in &kind.forms {
        check_reference(
            &known.business_forms,
            form.as_str(),
            "формы собственности",
            "business/forms/",
            report,
        );
    }
    let own: Vec<&BusinessForm> = kind
        .forms
        .iter()
        .filter_map(|form| forms.get(form.as_str()).copied())
        .collect();
    for form in &own {
        if let Some(until) = form.until
            && !own.iter().any(|next| next.is_open_on(until))
        {
            report.at(
                report.file.line_of("forms"),
                format!(
                    "с {} у дела нет формы собственности: «{}» закрывается, а следующей нет",
                    until.to_iso(),
                    form.title
                ),
            );
        }
    }
    if !kind.point_cost.is_positive() {
        report.at(
            report.file.line_of("point_cost"),
            "точка должна чего-то стоить: point_cost больше нуля".to_owned(),
        );
    }
    if kind.max_points == 0 {
        report.at(
            report.file.line_of("max_points"),
            "у дела должна быть хотя бы одна точка".to_owned(),
        );
    }
    for (key, amount) in [
        ("rent", kind.rent),
        ("wages", kind.wages),
        ("income", kind.income),
    ] {
        if amount.is_negative() {
            report.at(
                report.file.line_of(key),
                format!("{key} не может быть меньше нуля"),
            );
        }
    }
    for (good, quantity) in &kind.goods {
        if !known.goods.contains(good.as_str()) {
            report.at(
                report.file.line_of(good.as_str()),
                format!("товара {good} нет в goods/"),
            );
        }
        if *quantity == 0 {
            report.at(
                report.file.line_of(good.as_str()),
                format!("{good}: продажи в ноль штук"),
            );
        }
    }
    let margin_in_range = kind.margin > 0.0 && kind.margin <= 1.0;
    if !kind.goods.is_empty() && !margin_in_range {
        report.at(
            report.file.line_of("margin"),
            format!(
                "margin = {}: доля продаж, которая остаётся делу, — больше нуля и не больше 1",
                kind.margin
            ),
        );
    }
    if kind.goods.is_empty() && !kind.income.is_positive() {
        report.at(
            None,
            "дело ничего не приносит: нет ни товара goods, ни оборота income".to_owned(),
        );
    }
    let mut listed = BTreeSet::new();
    for policy in &kind.policies {
        if !listed.insert(*policy) {
            report.at(
                report.file.line_of("policies"),
                format!("политика {} указана дважды", policy.id()),
            );
        }
        if !known.policies.contains_key(policy.id()) {
            report.at(
                report.file.line_of("policies"),
                format!("политики {} нет в business/policies/", policy.id()),
            );
        }
    }
}

fn check_chance(key: &str, chance: f64, line: Option<usize>, report: &mut Report<'_, '_>) {
    if !(0.0..=1.0).contains(&chance) {
        report.at(line, format!("{key} = {chance}: доля от 0 до 1"));
    }
}
