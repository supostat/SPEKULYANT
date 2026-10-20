//! Цены от знакомых и газета (§9, §13.3): знакомый из другого города без своего повода за услугу
//! диктует цены своей толкучки, и они ложатся в цены по городам; раз в неделю газета называет самый
//! большой сдвиг цены за неделю с настоящими ценниками обеих недель, а дефицит или завал — по имени.

use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{CityId, Content, GoodId, NpcId};
use speculant_core::economy::{FactorRange, WEEK_DAYS};
use speculant_core::view::ContactView;
use speculant_core::{Command, Game, LifeSettings};

const NPC: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/npc");
const DAY_MILLISECONDS: u32 = 1800;
const NEWSPAPER: &str = "Газета: ";

/// Фикстура соперников без соперников и событий дня: в ленте только то, что проверяется.
fn quiet() -> Content {
    let mut content = Content::load(Path::new(NPC)).expect("фикстура соперников валидна");
    content.rival_rules.min_rivals = 0;
    content.rival_rules.max_rivals = 0;
    content.event_rules.daily_chance = 0.0;
    content
}

/// Тихая фикстура, где Кофи живёт в городе `city`.
fn kofi_living_in(city: Option<&str>) -> Content {
    let mut content = quiet();
    content
        .npcs
        .get_mut(&NpcId::new("kofi"))
        .expect("Кофи в фикстуре")
        .city = city.map(CityId::new);
    content
}

/// Студент в Москве; Кофи у него среди связей.
fn start(content: Content) -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed: 7,
    };
    Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается")
}

fn live(game: &mut Game, days: u32) {
    for _ in 0..days {
        assert!(game.advance(DAY_MILLISECONDS), "день проходит");
    }
}

fn call_kofi(game: &mut Game) {
    game.apply(Command::Call {
        contact: "kofi".to_owned(),
    })
    .expect("Кофи на связи");
}

fn last_message(game: &Game) -> String {
    game.view().messages.last().cloned().unwrap_or_default()
}

fn kofi(game: &Game) -> ContactView {
    game.view()
        .contacts
        .into_iter()
        .find(|contact| contact.contact == "kofi")
        .expect("Кофи среди связей")
}

/// День, когда игрок в последний раз видел ленинградскую толкучку, — одинаковый у всех товаров.
fn leningrad_seen(game: &Game) -> Option<String> {
    game.view().market.goods[0]
        .cities
        .iter()
        .find(|city| city.name == "Ленинград")
        .expect("Ленинград в ценах по городам")
        .seen
        .clone()
}

fn newspapers(game: &Game) -> Vec<String> {
    game.view()
        .messages
        .into_iter()
        .filter(|message| message.starts_with(NEWSPAPER))
        .collect()
}

/// Сдвиг цены покупки каждой пары «город × товар» за последнюю неделю истории — модуль логарифма
/// отношения — и строка газеты, какой она была бы об этой паре.
fn weekly_shifts(game: &Game) -> Vec<(f64, String)> {
    let content = game.content();
    let rules = &content.money;
    let today = game.today();
    let mut shifts = Vec::new();
    for (city_id, city) in &content.cities {
        if !city.is_open_on(today) {
            continue;
        }
        for (good_id, good) in &content.goods {
            let Some([before, now]) = game.price_history().last_two_weeks(city_id, good_id) else {
                continue;
            };
            let (Some(before_kopecks), Some(now_kopecks)) = (before.real_kopecks, now.real_kopecks)
            else {
                continue;
            };
            let tag = |kopecks: u32, date| {
                rules.format_money(rules.price_tag(f64::from(kopecks), date), date)
            };
            let swing = game
                .markets()
                .shocks()
                .iter()
                .find(|shock| {
                    shock.city.as_ref() == Some(city_id)
                        && shock.good == *good_id
                        && !shock.is_over_on(today)
                })
                .map_or("", |shock| {
                    if shock.factor > 1.0 {
                        ", дефицит"
                    } else {
                        ", завал"
                    }
                });
            let line = format!(
                "{NEWSPAPER}{}{swing} — {}: {}, неделю назад {}",
                city.name,
                good.name,
                tag(now_kopecks, now.date),
                tag(before_kopecks, before.date)
            );
            let size = (f64::from(now_kopecks) / f64::from(before_kopecks))
                .ln()
                .abs();
            shifts.push((size, line));
        }
    }
    shifts
}

/// Газета вышла сегодня и пишет о паре с самым большим сдвигом за неделю, с её ценниками.
fn assert_todays_newspaper_is_about_the_largest_shift(game: &Game) {
    let newspaper = newspapers(game).pop().expect("газета вышла");
    let shifts = weekly_shifts(game);
    let (reported, _) = shifts
        .iter()
        .find(|(_, line)| *line == newspaper)
        .unwrap_or_else(|| panic!("«{newspaper}» — не строка ни об одной паре"));
    assert!(
        shifts.iter().all(|(size, _)| size <= reported),
        "«{newspaper}» — не самый большой сдвиг недели"
    );
}

#[test]
fn a_contact_in_another_city_dictates_its_prices_for_a_favour() {
    let mut game = start(kofi_living_in(Some("leningrad")));
    live(&mut game, 3);
    assert_eq!(leningrad_seen(&game), None, "в Ленинграде игрок не бывал");
    call_kofi(&mut game);
    assert_eq!(last_message(&game), "Цены по телефону: Ленинград — Кофи");
    let today = game.today();
    let leningrad = CityId::new("leningrad");
    let money = &game.content().money;
    for row in game.view().market.goods {
        let seen = row
            .cities
            .iter()
            .find(|city| city.name == "Ленинград")
            .expect("Ленинград в ценах по городам");
        let quote = game
            .markets()
            .quote(game.content(), &leningrad, &GoodId::new(&row.good), today)
            .expect("товар продаётся");
        assert_eq!(seen.seen, Some(today.numeric()), "{}", row.name);
        assert_eq!(
            seen.buy,
            money.format_number(quote.buy, today),
            "{}",
            row.name
        );
        assert_eq!(
            seen.sell,
            money.format_number(quote.sell, today),
            "{}",
            row.name
        );
    }
    let kofi = kofi(&game);
    assert_eq!(
        kofi.debt.as_deref(),
        Some("За тобой услуг: 1"),
        "цены — услуга"
    );
    assert_eq!(
        kofi.role, "сокурсник из Ганы, Ленинград",
        "город знакомого виден"
    );
    assert_eq!(
        kofi.history.last().map(String::as_str),
        Some(format!("{}: Цены по телефону: Ленинград", today.numeric()).as_str())
    );
}

#[test]
fn a_contact_owed_too_much_refuses_and_tells_nothing_new() {
    let mut game = start(kofi_living_in(Some("leningrad")));
    let max_debt = game.content().reputation.max_debt;
    for _ in 0..max_debt {
        live(&mut game, 1);
        call_kofi(&mut game);
    }
    let seen = leningrad_seen(&game);
    live(&mut game, 1);
    call_kofi(&mut game);
    assert_eq!(
        last_message(&game),
        "Кофи: за тобой и так слишком много услуг"
    );
    assert_eq!(leningrad_seen(&game), seen, "цены прошлого звонка");
}

#[test]
fn a_contact_without_a_city_or_from_the_players_city_does_not_answer() {
    for city in [None, Some("moscow")] {
        let mut game = start(kofi_living_in(city));
        live(&mut game, 1);
        call_kofi(&mut game);
        assert_eq!(last_message(&game), "Кофи не берёт трубку", "{city:?}");
        assert_eq!(kofi(&game).debt, None, "{city:?}: услуги не было");
    }
}

#[test]
fn once_a_week_the_newspaper_tells_the_largest_shift_with_real_tags() {
    let mut game = start(quiet());
    live(&mut game, WEEK_DAYS - 1);
    assert!(
        newspapers(&game).is_empty(),
        "первой неделе сравнивать не с чем"
    );
    live(&mut game, 1);
    assert_eq!(newspapers(&game).len(), 1);
    assert_todays_newspaper_is_about_the_largest_shift(&game);
    live(&mut game, 2 * WEEK_DAYS);
    assert_eq!(newspapers(&game).len(), 3, "по газете в неделю");
    assert_todays_newspaper_is_about_the_largest_shift(&game);
}

#[test]
fn the_newspaper_calls_a_deficit_by_its_name() {
    let mut content = quiet();
    content.swings.daily_chance = 1.0;
    content.swings.deficit_share = 1.0;
    content.swings.deficit_factor = FactorRange { from: 3.0, to: 3.0 };
    let mut game = start(content);
    live(&mut game, WEEK_DAYS);
    assert_todays_newspaper_is_about_the_largest_shift(&game);
    let newspaper = newspapers(&game).pop().expect("газета вышла");
    assert!(newspaper.contains(", дефицит — "), "{newspaper}");
}
