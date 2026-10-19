//! Проверки контента после разбора: ссылки, флаги, окна дат, кривые, выборы и подстановки.

mod business;

use std::collections::{BTreeMap, BTreeSet};

use super::loader::{Entry, Parsed};
use super::{
    Biography, BusinessChange, Choice, City, Conditions, ContentError, ContentFile, EpochProfile,
    Event, Faction, Good, MoneyPlace, NpcTemplate,
};
use crate::economy::{MarketRules, MoneyRules, RubleCurve};
use crate::events::EventRules;
use crate::npc::RivalRules;
use crate::reputation::ReputationRules;
use crate::time::{GameDate, TimeRules};

const PLACEHOLDERS: [&str; 5] = ["npc", "price", "sum", "business", "manager"];
const NPC_PLACEHOLDER: &str = "npc";
const PRICE_PLACEHOLDER: &str = "price";
const SUM_PLACEHOLDER: &str = "sum";
const BUSINESS_PLACEHOLDERS: [&str; 2] = ["business", "manager"];
const FIRST_YEAR: i32 = 1984;
/// Индекс цен 1984 года — единица: реальные цены считаются в рублях 1984 года.
const BASE_INDEX: f64 = 1.0;
const ENTIRE_DEPOSIT_PERCENT: f64 = -100.0;

/// Множества идентификаторов, на которые можно ссылаться.
struct Known {
    epochs: BTreeSet<String>,
    goods: BTreeSet<String>,
    categories: BTreeSet<String>,
    cities: BTreeSet<String>,
    npcs: BTreeSet<String>,
    factions: BTreeSet<String>,
    events: BTreeSet<String>,
    road_events: BTreeSet<String>,
    flags_set: BTreeSet<String>,
    /// Предел уровней репутации, если правила разобрались.
    reputation_limit: Option<i32>,
    business_forms: BTreeSet<String>,
    habits: BTreeSet<String>,
    /// Подписи вариантов каждой политики.
    policies: BTreeMap<String, BTreeSet<String>>,
    /// Шкала черт сотрудников, если правила дел разобрались.
    trait_scale: Option<u8>,
}

/// Чем событие наполняет подстановки: участником, товаром для цены, суммой, делом происшествия и
/// его управляющим.
#[derive(Clone, Copy)]
struct Sources {
    npc: bool,
    price: bool,
    sum: bool,
    business: bool,
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
        factions: ids(&parsed.factions),
        events: ids(&parsed.events),
        road_events: parsed
            .events
            .iter()
            .filter(|entry| entry.value.road)
            .map(|entry| entry.id.clone())
            .collect(),
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
        reputation_limit: parsed
            .reputation
            .as_ref()
            .map(|(_, rules)| i32::from(rules.limit)),
        business_forms: ids(&parsed.business_forms),
        habits: ids(&parsed.habits),
        policies: parsed
            .policies
            .iter()
            .map(|entry| {
                let labels = entry
                    .value
                    .options
                    .iter()
                    .map(|option| option.label.clone())
                    .collect();
                (entry.id.clone(), labels)
            })
            .collect(),
        trait_scale: parsed
            .business_rules
            .as_ref()
            .map(|(_, rules)| rules.trait_scale),
    };
    business::validate(parsed, &known, errors);
    for entry in &parsed.epochs {
        let mut report = Report {
            file: entry.file,
            errors: &mut *errors,
        };
        if !known.epochs.contains(&entry.id) {
            report.at(
                None,
                format!("эпохи {} нет в календаре epochs/time.toml", entry.id),
            );
        }
        check_epoch_profile(&entry.value, &known, &mut report);
    }
    let described = ids(&parsed.epochs);
    for epoch in time.epochs.epochs() {
        if !described.contains(&epoch.id) {
            errors.push(ContentError::new(
                &format!("epochs/{}.toml", epoch.id),
                None,
                format!("у эпохи {} из календаря нет описания", epoch.id),
            ));
        }
    }
    if let Some((file, rules)) = &parsed.rival_rules {
        check_rival_rules(
            rules,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
    }
    if let Some((file, rules)) = &parsed.event_rules {
        check_event_rules(
            *rules,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
    }
    if let Some((file, rules)) = &parsed.reputation {
        check_reputation_rules(
            *rules,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.factions {
        check_faction(
            &entry.value,
            &known,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    let rival_rules = parsed.rival_rules.as_ref().map(|(_, rules)| rules);
    for entry in &parsed.npcs {
        check_npc(
            &entry.value,
            &known,
            rival_rules,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
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
        check_biography(
            &entry.value,
            &known,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    if let Some((file, rules)) = &parsed.market {
        check_market(
            rules,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
    }
    if let Some((file, rules)) = &parsed.money {
        check_money(
            rules,
            &mut Report {
                file,
                errors: &mut *errors,
            },
        );
    }
    for entry in &parsed.money_places {
        check_money_place(
            &entry.value,
            &mut Report {
                file: entry.file,
                errors: &mut *errors,
            },
        );
    }
    let routed: BTreeSet<&str> = parsed
        .routes
        .iter()
        .flat_map(|(_, routes)| routes.iter())
        .flat_map(|route| route.road_events.iter())
        .map(|road| road.event.as_str())
        .collect();
    for entry in &parsed.events {
        if entry.value.road && !routed.contains(entry.id.as_str()) {
            Report {
                file: entry.file,
                errors: &mut *errors,
            }
            .at(
                entry.file.line_of("road"),
                format!(
                    "дорожное событие {} не стоит ни на одном маршруте routes.toml",
                    entry.id
                ),
            );
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

fn check_good(entry: &Entry<'_, Good>, report: &mut Report<'_, '_>) {
    if entry.value.stock == 0 {
        report.at(
            report.file.line_of("stock"),
            "обычный запас товара на толкучке должен быть больше нуля".to_owned(),
        );
    }
    let curve = entry.value.curve.points();
    let curve_line = report.file.line_of("curve");
    if curve.is_empty() {
        report.at(
            curve_line,
            "у товара нет ни одной опорной точки цены".to_owned(),
        );
    }
    for point in curve {
        check_year(point.year, curve_line, report);
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

fn check_city(entry: &Entry<'_, City>, known: &Known, report: &mut Report<'_, '_>) {
    if entry.value.market.trim().is_empty() {
        report.at(
            report.file.line_of("market"),
            "у города нет толкучки: market пустой".to_owned(),
        );
    }
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

fn check_epoch_profile(profile: &EpochProfile, known: &Known, report: &mut Report<'_, '_>) {
    check_reference(
        &known.factions,
        profile.threat.faction.as_str(),
        "фракции угрозы",
        "factions/",
        report,
    );
    if profile.threat.label.trim().is_empty() {
        report.at(
            report.file.line_of("label"),
            "у угрозы эпохи нет подписи для «Кармана»".to_owned(),
        );
    }
    if profile.reputation_drift_days == 0 {
        report.at(
            report.file.line_of("reputation_drift_days"),
            "дрейф репутации раз в ноль дней невозможен".to_owned(),
        );
    }
}

fn check_event_rules(rules: EventRules, report: &mut Report<'_, '_>) {
    for (key, chance) in [
        ("daily_chance", rules.daily_chance),
        ("road_daily_chance", rules.road_daily_chance),
    ] {
        if !(0.0..=1.0).contains(&chance) {
            report.at(
                report.file.line_of(key),
                format!("{key} = {chance}: вероятность от 0 до 1"),
            );
        }
    }
}

fn check_reputation_rules(rules: ReputationRules, report: &mut Report<'_, '_>) {
    if rules.limit == 0 {
        report.at(
            report.file.line_of("limit"),
            "предел репутации должен быть больше нуля".to_owned(),
        );
    }
    if rules.max_debt == 0 {
        report.at(
            report.file.line_of("max_debt"),
            "без права хотя бы на одну услугу связи бесполезны: max_debt больше нуля".to_owned(),
        );
    }
}

fn check_faction(faction: &Faction, known: &Known, report: &mut Report<'_, '_>) {
    if let Some(epoch) = &faction.from_epoch {
        check_reference(
            &known.epochs,
            epoch.as_str(),
            "эпохи",
            "epochs/time.toml",
            report,
        );
    }
}

fn check_npc(
    npc: &NpcTemplate,
    known: &Known,
    rival_rules: Option<&RivalRules>,
    report: &mut Report<'_, '_>,
) {
    check_reference(
        &known.factions,
        npc.faction.as_str(),
        "фракции",
        "factions/",
        report,
    );
    match (npc.strategy, npc.capital) {
        (Some(strategy), Some(capital)) => {
            if !capital.is_positive() {
                report.at(
                    report.file.line_of("capital"),
                    "капитал соперника должен быть больше нуля".to_owned(),
                );
            }
            if rival_rules.is_some_and(|rules| !rules.strategies.contains_key(&strategy)) {
                report.at(
                    report.file.line_of("strategy"),
                    format!(
                        "у стратегии {} нет параметров в rivals.toml",
                        strategy.title()
                    ),
                );
            }
        }
        (Some(_), None) => report.at(
            report.file.line_of("strategy"),
            "у соперника со стратегией нет капитала capital".to_owned(),
        ),
        (None, Some(_)) => report.at(
            report.file.line_of("capital"),
            "капитал бывает только у соперника со стратегией".to_owned(),
        ),
        (None, None) => {}
    }
    if let Some(traits) = &npc.employee {
        if npc.strategy.is_some() {
            report.at(
                report.file.line_of("[employee]"),
                "соперник со стратегией не нанимается: employee только у людей без strategy"
                    .to_owned(),
            );
        }
        business::check_employee(traits, known, report);
    }
}

fn check_rival_rules(rules: &RivalRules, report: &mut Report<'_, '_>) {
    if rules.min_rivals > rules.max_rivals {
        report.at(
            report.file.line_of("min_rivals"),
            "соперников не меньше, чем больше: min_rivals выше max_rivals".to_owned(),
        );
    }
    if rules.prison_days == 0 {
        report.at(
            report.file.line_of("prison_days"),
            "срок в зоне не может быть нулевым".to_owned(),
        );
    }
    if rules.ruin_below.is_negative() {
        report.at(
            report.file.line_of("ruin_below"),
            "порог разорения не может быть меньше нуля".to_owned(),
        );
    }
    for (strategy, parameters) in &rules.strategies {
        let line = report
            .file
            .line_of(&format!("strategies.{}", strategy_key(*strategy)));
        let chances = [
            ("trade_chance", parameters.trade_chance),
            ("prison_chance", parameters.prison_chance),
            ("death_chance", parameters.death_chance),
            ("deputy_chance", parameters.deputy_chance),
            ("business_chance", parameters.business_chance),
        ];
        for (key, chance) in chances {
            if !(0.0..=1.0).contains(&chance) {
                report.at(
                    line,
                    format!(
                        "{}: {key} = {chance} — вероятность от 0 до 1",
                        strategy.title()
                    ),
                );
            }
        }
        if strategy.trades() && parameters.lot == 0 {
            report.at(
                line,
                format!("{} торгует лотами больше нуля: lot", strategy.title()),
            );
        }
        if parameters.daily_income.is_negative() || parameters.annual_percent < 0.0 {
            report.at(
                line,
                format!(
                    "{}: доход и процент не бывают меньше нуля",
                    strategy.title()
                ),
            );
        }
    }
}

/// Ключ стратегии в данных — так, как его пишет автор.
fn strategy_key(strategy: crate::npc::Strategy) -> String {
    serde_json::to_value(strategy)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .expect("стратегия пишется строкой")
}

fn check_biography(biography: &Biography, known: &Known, report: &mut Report<'_, '_>) {
    let city = biography.start_city.as_str();
    if !known.cities.contains(city) {
        report.at_reference(city, format!("стартового города {city} нет в cities/"));
    }
    for npc in &biography.contacts {
        check_reference(&known.npcs, npc.as_str(), "знакомого", "npc/", report);
    }
    if biography.start_cash.is_negative() {
        report.at(
            report.file.line_of("start_cash"),
            "стартовые наличные не могут быть меньше нуля".to_owned(),
        );
    }
}

fn check_market(rules: &MarketRules, report: &mut Report<'_, '_>) {
    let shares = [
        (
            "spread",
            rules.spread,
            0.0 < rules.spread && rules.spread < 1.0,
            "больше 0 и меньше 1",
        ),
        (
            "recovery",
            rules.recovery,
            0.0 < rules.recovery && rules.recovery <= 1.0,
            "больше 0 и не больше 1",
        ),
        (
            "daily_noise",
            rules.daily_noise,
            (0.0..1.0).contains(&rules.daily_noise),
            "от 0 и меньше 1",
        ),
    ];
    for (key, value, valid, range) in shares {
        if !valid {
            report.at(
                report.file.line_of(key),
                format!("{key} = {value}: доля должна быть {range}"),
            );
        }
    }
}

fn check_money(rules: &MoneyRules, report: &mut Report<'_, '_>) {
    let denomination_line = report.file.line_of("denomination");
    if rules.denomination.factor <= 1 {
        report.at(
            denomination_line,
            "деноминация должна укрупнять рубль: factor больше 1".to_owned(),
        );
    }
    if rules.denomination.date < GameDate::START {
        report.at(
            denomination_line,
            "деноминация раньше начала игры 1984-01-01".to_owned(),
        );
    }
    for (key, curve) in [
        ("inflation", &rules.inflation),
        ("checks_rate", &rules.checks_rate),
        ("dollar_rate", &rules.dollar_rate),
    ] {
        check_ruble_curve(key, curve, report);
    }
    if rules
        .inflation
        .points()
        .first()
        .is_some_and(|point| point.rubles != BASE_INDEX)
    {
        report.at(
            report.file.line_of("inflation"),
            "индекс 1984 года — единица: реальные цены считаются в рублях 1984 года".to_owned(),
        );
    }
}

fn check_ruble_curve(key: &str, curve: &RubleCurve, report: &mut Report<'_, '_>) {
    let line = report.file.line_of(key);
    let points = curve.points();
    if points.first().is_none_or(|point| point.year != FIRST_YEAR) {
        report.at(
            line,
            format!("кривая {key} должна начинаться с {FIRST_YEAR} года"),
        );
    }
    for point in points {
        check_year(point.year, line, report);
        if !(point.rubles.is_finite() && point.rubles > 0.0) {
            report.at(
                line,
                format!(
                    "{key}: рублей в {} году должно быть больше нуля",
                    point.year
                ),
            );
        }
    }
    if points.windows(2).any(|pair| pair[1].year <= pair[0].year) {
        report.at(
            line,
            format!("годы опорных точек {key} должны строго возрастать"),
        );
    }
}

fn check_money_place(place: &MoneyPlace, report: &mut Report<'_, '_>) {
    for (key, date) in [
        ("opens", place.opens),
        ("closes", place.closes),
        ("illegal_until", place.illegal_until),
    ] {
        if date.is_some_and(|date| date < GameDate::START) {
            report.at(
                report.file.line_of(key),
                format!("{key} раньше начала игры 1984-01-01"),
            );
        }
    }
    if let (Some(opens), Some(closes)) = (place.opens, place.closes)
        && closes <= opens
    {
        report.at(
            report.file.line_of("closes"),
            "место закрывается не позже, чем открывается".to_owned(),
        );
    }
    let interest_line = report.file.line_of("interest");
    for point in &place.interest {
        check_year(point.year, interest_line, report);
        if !(point.percent.is_finite() && point.percent > ENTIRE_DEPOSIT_PERCENT) {
            report.at(
                interest_line,
                format!(
                    "процент {} года должен быть больше {ENTIRE_DEPOSIT_PERCENT}",
                    point.year
                ),
            );
        }
    }
    if place
        .interest
        .windows(2)
        .any(|pair| pair[1].year <= pair[0].year)
    {
        report.at(
            interest_line,
            "годы процентов должны строго возрастать".to_owned(),
        );
    }
}

/// Год опорной точки: не раньше начала игры и в пределах календаря.
fn check_year(year: i32, line: Option<usize>, report: &mut Report<'_, '_>) {
    if year < FIRST_YEAR {
        report.at(
            line,
            format!("опорная точка {year} раньше начала игры в {FIRST_YEAR} году"),
        );
    } else if GameDate::new_year(year).is_none() {
        report.at(line, format!("год {year} вне календаря игры"));
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
        for road in &route.road_events {
            let event = road.event.as_str();
            if !known.events.contains(event) {
                report.at_reference(event, format!("события {event} нет в events/"));
            } else if !known.road_events.contains(event) {
                report.at_reference(
                    event,
                    format!("событие {event} стоит на маршруте, но не помечено road = true"),
                );
            }
            if road.weight == 0 {
                report.at_reference(
                    event,
                    format!("вес дорожного события {event} должен быть больше нуля"),
                );
            }
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
    if event.cooldown_days == Some(0) {
        report.at(
            report.file.line_of("cooldown_days"),
            "отдых события в ноль дней — это каждый день; убери cooldown_days или задай срок"
                .to_owned(),
        );
    }
    if let Some(good) = &event.price_of {
        check_reference(&known.goods, good.as_str(), "товара", "goods/", report);
    }
    if event.sum.is_some_and(|sum| !sum.is_positive()) {
        report.at(
            report.file.line_of("sum"),
            "сумма события должна быть больше нуля".to_owned(),
        );
    }
    if event.incident.is_some() {
        let line = report.file.line_of("incident");
        if event.road {
            report.at(
                line,
                "событие происшествия с делом не бывает дорожным".to_owned(),
            );
        }
        if event.epoch.is_none() {
            report.at(
                line,
                "в зоне дел нет: событие происшествия — только в events/<эпоха>/".to_owned(),
            );
        }
    }
    check_conditions(&event.conditions, known, report);
    let sources = Sources {
        npc: event.conditions.npc.is_some() || event.conditions.rival.is_some(),
        price: event.price_of.is_some(),
        sum: event.sum.is_some(),
        business: event.incident.is_some(),
    };
    check_placeholders(&event.text, sources, report);
    let has_rival = event.conditions.rival.is_some();
    for (index, choice) in event.choices.iter().enumerate() {
        check_choice(choice, index, sources, known, report);
        if choice.conditions.rival.is_some() {
            report.at(
                report.file.nth_line_of("[[choices]]", index),
                format!(
                    "выбор {}: соперник выбирается для всего события, условие rival — в [conditions]",
                    index + 1
                ),
            );
        }
        if choice.consequences.rival.is_some() && !has_rival {
            report.at(
                report.file.nth_line_of("[[choices]]", index),
                format!(
                    "выбор {}: последствие rival без соперника в условиях события",
                    index + 1
                ),
            );
        }
        if choice.consequences.business.is_some() && event.incident.is_none() {
            report.at(
                report.file.nth_line_of("[[choices]]", index),
                format!(
                    "выбор {}: последствие business бывает только у события происшествия incident",
                    index + 1
                ),
            );
        }
    }
}

fn check_choice(
    choice: &Choice,
    index: usize,
    sources: Sources,
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
    check_placeholders(&choice.text, sources, report);
    if let Some(outcome) = &choice.outcome {
        check_placeholders(outcome, sources, report);
    }
    let consequences = &choice.consequences;
    for change in &consequences.reputation {
        check_reference(
            &known.factions,
            change.faction.as_str(),
            "фракции",
            "factions/",
            report,
        );
    }
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
        if !change.changes_anything() {
            report.at_reference(
                change.npc.as_str(),
                format!("последствие для {} ничего не меняет", change.npc),
            );
        }
        if change.away_days == Some(0) {
            report.at(
                report.file.line_of("away_days"),
                format!("{} не может уехать на ноль дней", change.npc),
            );
        }
    }
    if let Some(change) = &consequences.business {
        check_business_change(change, index, known, report);
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

/// Перемена в деле: варианты политик по подписям из данных, закрытие на срок больше нуля, и
/// что-то в деле должно меняться.
fn check_business_change(
    change: &BusinessChange,
    index: usize,
    known: &Known,
    report: &mut Report<'_, '_>,
) {
    let choice_line = report.file.nth_line_of("[[choices]]", index);
    if !change.changes_anything() {
        report.at(
            choice_line,
            format!("выбор {}: последствие business ничего не меняет", index + 1),
        );
    }
    if change.closed_days == Some(0) {
        report.at(
            report.file.line_of("closed_days"),
            format!("выбор {}: закрыть дело на ноль дней нельзя", index + 1),
        );
    }
    for setting in &change.policies {
        let policy = setting.policy.id();
        match known.policies.get(policy) {
            None => report.at(
                report.file.line_of(policy),
                format!("политики {policy} нет в business/policies/"),
            ),
            Some(labels) if !labels.contains(&setting.option) => report.at(
                report.file.line_of(&format!("\"{}\"", setting.option)),
                format!("у политики {policy} нет варианта «{}»", setting.option),
            ),
            Some(_) => {}
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
        check_reference(
            &known.factions,
            reputation.faction.as_str(),
            "фракции",
            "factions/",
            report,
        );
        let reputation_line = report.file.line_of(&format!("\"{}\"", reputation.faction));
        if let Some(limit) = known.reputation_limit {
            for bound in [reputation.at_least, reputation.at_most]
                .into_iter()
                .flatten()
            {
                if bound.abs() > limit {
                    report.at(
                        reputation_line,
                        format!(
                            "граница репутации {} {bound} за пределом ±{limit}",
                            reputation.faction
                        ),
                    );
                }
            }
        }
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

/// Подстановки `{имя}` в тексте: только известные и только те, которым событие даёт значение —
/// `{npc}` при условии `npc`, `{price}` при `price_of`, `{sum}` при `sum`, `{business}` и
/// `{manager}` у события происшествия.
fn check_placeholders(text: &str, sources: Sources, report: &mut Report<'_, '_>) {
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
                format!(
                    "подстановки {{{name}}} нет, есть {{npc}}, {{price}}, {{sum}}, {{business}}, {{manager}}"
                ),
            );
        } else if name == NPC_PLACEHOLDER && !sources.npc {
            report.at(
                line,
                "подстановка {npc} в событии без условия npc".to_owned(),
            );
        } else if name == PRICE_PLACEHOLDER && !sources.price {
            report.at(
                line,
                "подстановка {price} в событии без price_of".to_owned(),
            );
        } else if name == SUM_PLACEHOLDER && !sources.sum {
            report.at(line, "подстановка {sum} в событии без sum".to_owned());
        } else if BUSINESS_PLACEHOLDERS.contains(&name) && !sources.business {
            report.at(
                line,
                format!("подстановка {{{name}}} в событии без происшествия incident"),
            );
        }
        rest = &after[end + 1..];
    }
}
