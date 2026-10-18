//! Контент как данные: минимальный набор читается целиком, а набор с ошибками отвергается
//! с местом каждой ошибки «файл:строка».

use std::path::Path;

use speculant_core::Content;
use speculant_core::content::{ContentError, ContentFile, EpochId, EventId, GoodId};
use speculant_core::economy::Real;
use speculant_core::time::GameDate;

const CONTENT_MIN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/content-min");
const CONTENT_BROKEN: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/content-broken");

fn load_min() -> Content {
    Content::load(Path::new(CONTENT_MIN)).expect("минимальный набор валиден")
}

fn errors_of(files: Vec<ContentFile>) -> Vec<String> {
    Content::from_files(files)
        .expect_err("набор должен быть отвергнут")
        .iter()
        .map(ContentError::to_string)
        .collect()
}

#[test]
fn the_minimal_set_loads_every_kind_of_content() {
    let content = load_min();
    assert_eq!(content.time.epochs.epochs().len(), 1);
    assert_eq!(content.epochs.len(), 1);
    assert_eq!(content.goods.len(), 3);
    assert_eq!(content.cities.len(), 2);
    assert_eq!(content.routes.len(), 2);
    assert_eq!(content.money_places.len(), 1);
    assert_eq!(content.factions.len(), 3);
    assert_eq!(content.reputation.limit, 7);
    assert_eq!(content.money.denomination.factor, 1000);
    assert!((content.market.spread - 0.08).abs() < f64::EPSILON);
    assert_eq!(content.biographies.len(), 1);
    assert_eq!(content.npcs.len(), 1);
    assert_eq!(content.events.len(), 2);
}

#[test]
fn prices_in_rubles_of_1984_become_kopecks() {
    let content = load_min();
    assert_eq!(
        content.goods[&GoodId::new("jeans")].curve.points()[0].price,
        Real::from_kopecks(18_000)
    );
    assert_eq!(
        content.goods[&GoodId::new("chewing_gum")].curve.points()[0].price,
        Real::from_kopecks(2_550)
    );
}

#[test]
fn the_section_10_2_event_keeps_its_conditions_choices_and_chain() {
    let content = load_min();
    let event = &content.events[&EventId::new("vodka_crate")];
    assert_eq!(event.epoch, Some(EpochId::new("stagnation")));
    assert_eq!(event.choices.len(), 3);
    let window = event.conditions.window.expect("у события есть окно дат");
    assert_eq!(
        window.from,
        GameDate::from_ymd(1985, 5, 17).expect("дата существует")
    );
    assert_eq!(
        event.conditions.money.map(|money| money.at_least),
        Some(Real::from_kopecks(60_000))
    );
    let snitch = &event.choices[2];
    assert_eq!(snitch.consequences.set_flags[0].as_str(), "snitched_1985");
    assert_eq!(
        snitch.follow_ups[0].event,
        EventId::new("vodka_man_returns")
    );
    assert!((snitch.follow_ups[0].chance - 0.5).abs() < f64::EPSILON);
}

#[test]
fn the_broken_set_reports_every_defect_with_its_place() {
    let errors: Vec<String> = Content::load(Path::new(CONTENT_BROKEN))
        .expect_err("набор с ошибками отвергнут")
        .iter()
        .map(ContentError::to_string)
        .collect();
    let expected = [
        "biographies/student.toml:3: стартового города kiev нет в cities/",
        "cities/leningrad.toml:4: unknown field `population`",
        "cities/moscow.toml:4: категории electronics нет ни у одного товара",
        "economy/market.toml:1: spread = 1.5: доля должна быть больше 0 и меньше 1",
        "economy/money.toml:2: индекс 1984 года — единица",
        "events/perestroika/cooperative.toml: каталога events/perestroika/ нет среди эпох календаря",
        "events/stagnation/broken.toml:3: подстановка {npc} в событии без условия npc",
        "events/stagnation/broken.toml:3: подстановки {bribe} нет",
        "events/stagnation/broken.toml:6: окно дат начинается 1983-06-01 — раньше начала игры",
        "events/stagnation/broken.toml:7: города kiev нет в cities/",
        "events/stagnation/broken.toml:8: флаг never_set_flag не ставит ни одно последствие",
        "events/stagnation/broken.toml:10: выбор 1 без последствий, исхода и продолжения",
        "events/stagnation/broken.toml:17: события-продолжения missing_event нет в events/",
        "events/stagnation/greedy.toml:3: отдых события в ноль дней — это каждый день",
        "events/stagnation/lost_luggage.toml:3: дорожное событие lost_luggage не стоит ни на одном маршруте",
        "events/stagnation/greedy.toml:4: подстановка {price} в событии без price_of",
        "events/stagnation/greedy.toml:4: подстановка {sum} в событии без sum",
        "events/stagnation/greedy.toml:10: последствие для vodka_man ничего не меняет",
        "events/stagnation/vodka_crate.toml:37: фракции police нет в factions/",
        "goods/Black-Caviar.toml: имя файла «Black-Caviar» — не идентификатор",
        "goods/jeans.toml:6: годы опорных точек кривой должны строго возрастать",
        "goods/sugar.toml:1: invalid basic string",
        "money/pyramids.toml:4: место закрывается не позже, чем открывается",
        "music/song.toml: путь не входит в раскладку data/",
        "routes.toml:3: города riga нет в cities/",
        "routes.toml:6: событие vodka_crate стоит на маршруте, но не помечено road = true",
    ];
    for prefix in expected {
        assert!(
            errors.iter().any(|error| error.starts_with(prefix)),
            "нет ошибки «{prefix}…» среди {errors:#?}"
        );
    }
    assert_eq!(errors.len(), expected.len(), "лишние ошибки: {errors:#?}");
}

#[test]
fn an_event_id_used_twice_is_refused() {
    let mut files = Content::read_files(Path::new(CONTENT_MIN)).expect("набор читается");
    let crate_event = files
        .iter()
        .find(|file| file.path == "events/stagnation/vodka_crate.toml")
        .expect("событие есть")
        .clone();
    files.push(ContentFile::new(
        "zone/events/vodka_crate.toml",
        crate_event.text,
    ));
    let errors = errors_of(files);
    assert!(
        errors
            .iter()
            .any(|error| error.contains("событие vodka_crate описано в двух файлах")),
        "{errors:#?}"
    );
}

#[test]
fn zone_events_load_without_an_epoch() {
    let mut files = Content::read_files(Path::new(CONTENT_MIN)).expect("набор читается");
    files.push(ContentFile::new(
        "zone/events/tea_trade.toml",
        "title = \"Чай на промзоне\"\nweight = 3\ntext = \"Пачка чая ходит по бараку.\"\n\n[[choices]]\ntext = \"Взять\"\noutcome = \"Чай твой.\"\n",
    ));
    let content = Content::from_files(files).expect("событие зоны валидно");
    assert_eq!(content.events[&EventId::new("tea_trade")].epoch, None);
}

#[test]
fn content_without_a_calendar_is_refused() {
    let files: Vec<ContentFile> = Content::read_files(Path::new(CONTENT_MIN))
        .expect("набор читается")
        .into_iter()
        .filter(|file| file.path != "epochs/time.toml")
        .collect();
    let errors = errors_of(files);
    assert_eq!(
        errors,
        vec!["epochs/time.toml: нет календаря эпох и часов игры".to_owned()]
    );
}

#[test]
fn every_epoch_of_the_calendar_needs_a_description() {
    let files: Vec<ContentFile> = Content::read_files(Path::new(CONTENT_MIN))
        .expect("набор читается")
        .into_iter()
        .filter(|file| file.path != "epochs/stagnation.toml")
        .collect();
    assert_eq!(
        errors_of(files),
        vec!["epochs/stagnation.toml: у эпохи stagnation из календаря нет описания".to_owned()]
    );
}

#[test]
fn the_real_data_directory_is_valid() {
    let data = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
    Content::load(Path::new(data)).expect("data/ валиден");
}
