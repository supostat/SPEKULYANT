use std::collections::BTreeMap;

use serde::Deserialize;
use serde::de::DeserializeOwned;

use super::validation;
use super::{
    Biography, BiographyId, City, CityId, Content, ContentError, ContentFile, EpochId,
    EpochProfile, Event, EventId, Good, GoodId, NpcId, NpcTemplate, Route,
};
use crate::time::TimeRules;

const TIME_FILE: &str = "epochs/time.toml";
const ROUTES_FILE: &str = "routes.toml";
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
    pub epochs: Vec<Entry<'a, EpochProfile>>,
    pub goods: Vec<Entry<'a, Good>>,
    pub cities: Vec<Entry<'a, City>>,
    pub routes: Option<(&'a ContentFile, Vec<Route>)>,
    pub biographies: Vec<Entry<'a, Biography>>,
    pub npcs: Vec<Entry<'a, NpcTemplate>>,
    pub events: Vec<Entry<'a, Event>>,
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
    Ok(Content {
        time,
        epochs: keyed(&parsed.epochs, EpochId::new),
        goods: keyed(&parsed.goods, GoodId::new),
        cities: keyed(&parsed.cities, CityId::new),
        routes: parsed.routes.map(|(_, routes)| routes).unwrap_or_default(),
        biographies: keyed(&parsed.biographies, BiographyId::new),
        npcs: keyed(&parsed.npcs, NpcId::new),
        events,
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
        ["routes.toml"] => {
            if let Some(routes) = parse::<RoutesFile>(file, errors) {
                parsed.routes = Some((file, routes.routes));
            }
        }
        ["epochs", name] => push_entry(file, name, &mut parsed.epochs, errors),
        ["goods", name] => push_entry(file, name, &mut parsed.goods, errors),
        ["cities", name] => push_entry(file, name, &mut parsed.cities, errors),
        ["biographies", name] => push_entry(file, name, &mut parsed.biographies, errors),
        ["npc", name] => push_entry(file, name, &mut parsed.npcs, errors),
        ["events", epoch, name] => push_event(file, name, Some(EpochId::new(*epoch)), parsed, errors),
        ["zone", "events", name] => push_event(file, name, None, parsed, errors),
        _ => errors.push(ContentError::new(
            &file.path,
            None,
            format!("путь не входит в раскладку data/ (goods/, cities/, {ROUTES_FILE}, epochs/, biographies/, npc/, events/<эпоха>/, zone/events/)"),
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
