use std::collections::BTreeMap;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::validation;
use super::{
    Biography, BiographyId, BusinessForm, BusinessFormId, BusinessKind, BusinessKindId, City,
    CityId, Content, ContentError, ContentFile, Ending, EndingId, EpochId, EpochProfile, Event,
    EventId, Faction, FactionId, Good, GoodId, Habit, HabitId, Housing, HousingId, MoneyPlace,
    MoneyPlaceId, NpcId, NpcTemplate, Policy, Route, ZoneService, ZoneServiceId,
};
use crate::business::{BusinessRules, PolicyKind};
use crate::economy::{MarketRules, MoneyRules};
use crate::events::EventRules;
use crate::life::LifeRules;
use crate::npc::RivalRules;
use crate::reputation::ReputationRules;
use crate::time::TimeRules;
use crate::zone::ZoneRules;

const TIME_FILE: &str = "epochs/time.toml";
const MARKET_FILE: &str = "economy/market.toml";
const MONEY_FILE: &str = "economy/money.toml";
const ROUTES_FILE: &str = "routes.toml";
const REPUTATION_FILE: &str = "reputation.toml";
const EVENT_RULES_FILE: &str = "event_rules.toml";
const RIVALS_FILE: &str = "rivals.toml";
const BUSINESS_RULES_FILE: &str = "business/rules.toml";
const LIFE_FILE: &str = "life.toml";
const ZONE_RULES_FILE: &str = "zone/rules.toml";
const TOML_EXTENSION: &str = ".toml";

/// Сущность, прочитанная из файла: идентификатор из имени файла и сам файл для мест ошибок.
pub(super) struct Entry<'a, T> {
    pub file: &'a ContentFile,
    pub id: String,
    pub value: T,
}

/// Всё, что разобралось без синтаксических ошибок, — до проверки ссылок.
#[derive(Default)]
pub(super) struct Parsed<'a> {
    pub time: Option<TimeRules>,
    pub market: Option<(&'a ContentFile, MarketRules)>,
    pub money: Option<(&'a ContentFile, MoneyRules)>,
    pub reputation: Option<(&'a ContentFile, ReputationRules)>,
    pub event_rules: Option<(&'a ContentFile, EventRules)>,
    pub rival_rules: Option<(&'a ContentFile, RivalRules)>,
    pub business_rules: Option<(&'a ContentFile, BusinessRules)>,
    pub life: Option<(&'a ContentFile, LifeRules)>,
    pub zone: Option<(&'a ContentFile, ZoneRules)>,
    pub epochs: Vec<Entry<'a, EpochProfile>>,
    pub goods: Vec<Entry<'a, Good>>,
    pub cities: Vec<Entry<'a, City>>,
    pub routes: Option<(&'a ContentFile, Vec<Route>)>,
    pub money_places: Vec<Entry<'a, MoneyPlace>>,
    pub factions: Vec<Entry<'a, Faction>>,
    pub biographies: Vec<Entry<'a, Biography>>,
    pub housing: Vec<Entry<'a, Housing>>,
    pub endings: Vec<Entry<'a, Ending>>,
    pub npcs: Vec<Entry<'a, NpcTemplate>>,
    pub business_forms: Vec<Entry<'a, BusinessForm>>,
    pub business_kinds: Vec<Entry<'a, BusinessKind>>,
    pub policies: Vec<Entry<'a, Policy>>,
    pub habits: Vec<Entry<'a, Habit>>,
    pub events: Vec<Entry<'a, Event>>,
    pub zone_services: Vec<Entry<'a, ZoneService>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RoutesFile {
    routes: Vec<Route>,
}

pub(super) fn load(files: Vec<ContentFile>) -> Result<Content, Vec<ContentError>> {
    let mut errors = Vec::new();
    let mut parsed = Parsed::default();
    for file in &files {
        sort_into(file, &mut parsed, &mut errors);
    }
    let Some(time) = parsed.time.take() else {
        errors.push(ContentError::new(
            TIME_FILE,
            None,
            "нет календаря эпох и часов игры",
        ));
        errors.sort();
        return Err(errors);
    };
    validation::validate(&parsed, &time, &mut errors);
    if parsed.market.is_none() {
        errors.push(ContentError::new(MARKET_FILE, None, "нет правил толкучки"));
    }
    if parsed.money.is_none() {
        errors.push(ContentError::new(
            MONEY_FILE,
            None,
            "нет индекса цен и курсов валют",
        ));
    }
    if parsed.reputation.is_none() {
        errors.push(ContentError::new(
            REPUTATION_FILE,
            None,
            "нет правил репутации и связей",
        ));
    }
    if parsed.event_rules.is_none() {
        errors.push(ContentError::new(
            EVENT_RULES_FILE,
            None,
            "нет правил розыгрыша событий",
        ));
    }
    if parsed.rival_rules.is_none() {
        errors.push(ContentError::new(
            RIVALS_FILE,
            None,
            "нет правил соперников",
        ));
    }
    if parsed.business_rules.is_none() {
        errors.push(ContentError::new(
            BUSINESS_RULES_FILE,
            None,
            "нет правил дел",
        ));
    }
    if parsed.life.is_none() {
        errors.push(ContentError::new(LIFE_FILE, None, "нет правил жизни"));
    }
    if parsed.zone.is_none() {
        errors.push(ContentError::new(ZONE_RULES_FILE, None, "нет правил зоны"));
    }
    let mut events = BTreeMap::new();
    for entry in &parsed.events {
        if events
            .insert(EventId::new(&entry.id), entry.value.clone())
            .is_some()
        {
            errors.push(ContentError::new(
                &entry.file.path,
                None,
                format!("событие {} описано в двух файлах", entry.id),
            ));
        }
    }
    if !errors.is_empty() {
        errors.sort();
        return Err(errors);
    }
    let (
        Some((_, market)),
        Some((_, money)),
        Some((_, reputation)),
        Some((_, event_rules)),
        Some((_, rival_rules)),
        Some((_, business)),
        Some((_, life)),
        Some((_, zone)),
    ) = (
        parsed.market,
        parsed.money,
        parsed.reputation,
        parsed.event_rules,
        parsed.rival_rules,
        parsed.business_rules,
        parsed.life,
        parsed.zone,
    )
    else {
        unreachable!("без любого из файлов правил загрузка вернула ошибку выше");
    };
    Ok(Content {
        time,
        market,
        money,
        reputation,
        event_rules,
        rival_rules,
        business,
        life,
        zone,
        epochs: keyed(&parsed.epochs, EpochId::new),
        goods: keyed(&parsed.goods, GoodId::new),
        cities: keyed(&parsed.cities, CityId::new),
        routes: parsed.routes.map(|(_, routes)| routes).unwrap_or_default(),
        money_places: keyed(&parsed.money_places, MoneyPlaceId::new),
        factions: keyed(&parsed.factions, FactionId::new),
        biographies: keyed(&parsed.biographies, BiographyId::new),
        housing: keyed(&parsed.housing, HousingId::new),
        endings: keyed(&parsed.endings, EndingId::new),
        npcs: keyed(&parsed.npcs, NpcId::new),
        business_forms: keyed(&parsed.business_forms, BusinessFormId::new),
        business_kinds: keyed(&parsed.business_kinds, BusinessKindId::new),
        policies: parsed
            .policies
            .iter()
            .map(|entry| {
                let kind = PolicyKind::from_id(&entry.id)
                    .expect("файл политики с чужим именем загрузка отвергла выше");
                (kind, entry.value.clone())
            })
            .collect(),
        habits: keyed(&parsed.habits, HabitId::new),
        events,
        zone_services: keyed(&parsed.zone_services, ZoneServiceId::new),
    })
}

fn keyed<T: Clone, Id: Ord>(
    entries: &[Entry<'_, T>],
    make_id: fn(String) -> Id,
) -> BTreeMap<Id, T> {
    entries
        .iter()
        .map(|entry| (make_id(entry.id.clone()), entry.value.clone()))
        .collect()
}

/// Раскладывает файл по виду сущности по его пути в `data/`.
fn sort_into<'a>(file: &'a ContentFile, parsed: &mut Parsed<'a>, errors: &mut Vec<ContentError>) {
    let segments: Vec<&str> = file.path.split('/').collect();
    match segments.as_slice() {
        ["epochs", "time.toml"] => match TimeRules::from_toml(&file.text) {
            Ok(time) => parsed.time = Some(time),
            Err(error) => errors.push(ContentError::new(&file.path, None, error.to_string())),
        },
        ["economy", "market.toml"] => {
            if let Some(rules) = parse::<MarketRules>(file, errors) {
                parsed.market = Some((file, rules));
            }
        }
        ["economy", "money.toml"] => {
            if let Some(rules) = parse::<MoneyRules>(file, errors) {
                parsed.money = Some((file, rules));
            }
        }
        ["reputation.toml"] => {
            if let Some(rules) = parse::<ReputationRules>(file, errors) {
                parsed.reputation = Some((file, rules));
            }
        }
        ["event_rules.toml"] => {
            if let Some(rules) = parse::<EventRules>(file, errors) {
                parsed.event_rules = Some((file, rules));
            }
        }
        ["rivals.toml"] => {
            if let Some(rules) = parse::<RivalRules>(file, errors) {
                parsed.rival_rules = Some((file, rules));
            }
        }
        ["life.toml"] => {
            if let Some(rules) = parse::<LifeRules>(file, errors) {
                parsed.life = Some((file, rules));
            }
        }
        ["housing", name] => push_entry(file, name, &mut parsed.housing, errors),
        ["endings", name] => push_entry(file, name, &mut parsed.endings, errors),
        ["business", "rules.toml"] => {
            if let Some(rules) = parse::<BusinessRules>(file, errors) {
                parsed.business_rules = Some((file, rules));
            }
        }
        ["business", "forms", name] => push_entry(file, name, &mut parsed.business_forms, errors),
        ["business", "kinds", name] => push_entry(file, name, &mut parsed.business_kinds, errors),
        ["business", "habits", name] => push_entry(file, name, &mut parsed.habits, errors),
        ["business", "policies", name] => push_entry(file, name, &mut parsed.policies, errors),
        ["routes.toml"] => {
            if let Some(routes) = parse::<RoutesFile>(file, errors) {
                parsed.routes = Some((file, routes.routes));
            }
        }
        ["epochs", name] => push_entry(file, name, &mut parsed.epochs, errors),
        ["goods", name] => push_entry(file, name, &mut parsed.goods, errors),
        ["cities", name] => push_entry(file, name, &mut parsed.cities, errors),
        ["money", name] => push_entry(file, name, &mut parsed.money_places, errors),
        ["factions", name] => push_entry(file, name, &mut parsed.factions, errors),
        ["biographies", name] => push_entry(file, name, &mut parsed.biographies, errors),
        ["npc", name] => push_entry(file, name, &mut parsed.npcs, errors),
        ["events", epoch, name] => push_event(file, name, Some(EpochId::new(*epoch)), parsed, errors),
        ["zone", "rules.toml"] => {
            if let Some(rules) = parse::<ZoneRules>(file, errors) {
                parsed.zone = Some((file, rules));
            }
        }
        ["zone", "services", name] => push_entry(file, name, &mut parsed.zone_services, errors),
        ["zone", "events", name] => push_event(file, name, None, parsed, errors),
        _ => errors.push(ContentError::new(
            &file.path,
            None,
            format!("путь не входит в раскладку data/ (goods/, cities/, {ROUTES_FILE}, {MARKET_FILE}, {MONEY_FILE}, money/, {REPUTATION_FILE}, factions/, {EVENT_RULES_FILE}, {RIVALS_FILE}, {BUSINESS_RULES_FILE}, business/forms/, business/kinds/, business/policies/, business/habits/, {LIFE_FILE}, housing/, endings/, epochs/, biographies/, npc/, events/<эпоха>/, {ZONE_RULES_FILE}, zone/services/, zone/events/)"),
        )),
    }
}

fn push_entry<'a, T: DeserializeOwned>(
    file: &'a ContentFile,
    name: &str,
    entries: &mut Vec<Entry<'a, T>>,
    errors: &mut Vec<ContentError>,
) {
    let Some(id) = id_from_file_name(file, name, errors) else {
        return;
    };
    if let Some(value) = parse(file, errors) {
        entries.push(Entry { file, id, value });
    }
}

fn push_event<'a>(
    file: &'a ContentFile,
    name: &str,
    epoch: Option<EpochId>,
    parsed: &mut Parsed<'a>,
    errors: &mut Vec<ContentError>,
) {
    let Some(id) = id_from_file_name(file, name, errors) else {
        return;
    };
    if let Some(mut event) = parse::<Event>(file, errors) {
        event.epoch = epoch;
        parsed.events.push(Entry {
            file,
            id,
            value: event,
        });
    }
}

/// Идентификатор — имя файла без `.toml`: латиница в нижнем регистре, цифры и `_`, с буквы.
fn id_from_file_name(
    file: &ContentFile,
    name: &str,
    errors: &mut Vec<ContentError>,
) -> Option<String> {
    let Some(stem) = name.strip_suffix(TOML_EXTENSION) else {
        errors.push(ContentError::new(
            &file.path,
            None,
            "данные пишутся только в .toml",
        ));
        return None;
    };
    let valid = stem.starts_with(|first: char| first.is_ascii_lowercase())
        && stem.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        });
    if !valid {
        errors.push(ContentError::new(
            &file.path,
            None,
            format!("имя файла «{stem}» — не идентификатор: латиница в нижнем регистре, цифры и _, с буквы"),
        ));
        return None;
    }
    Some(stem.to_owned())
}

fn parse<T: DeserializeOwned>(file: &ContentFile, errors: &mut Vec<ContentError>) -> Option<T> {
    match toml::from_str(&file.text) {
        Ok(value) => Some(value),
        Err(error) => {
            let line = error.span().map(|span| file.line_at_offset(span.start));
            errors.push(ContentError::new(
                &file.path,
                line,
                error.message().trim().to_owned(),
            ));
            None
        }
    }
}
