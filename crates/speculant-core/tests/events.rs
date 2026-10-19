//! Система событий (§10): пример §10.2 с тремя исходами, условия и окна, однократные и
//! отдыхающие события, розыгрыш дня по весам и эпохам, цепочки продолжений, доступность выборов
//! и последствия для связей. Время на событии стоит.

use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{CityId, Content, EventId, FactionId, FlagId, GoodId, NpcId};
use speculant_core::time::GameDate;
use speculant_core::view::{ChoiceView, ContactView, EventView};
use speculant_core::{Command, CommandError, Game, LifeSettings};

const EVENTS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/events");
const DAY_MILLISECONDS: u32 = 1800;

fn content(daily_chance: f64) -> Arc<Content> {
    let mut content = Content::load(Path::new(EVENTS)).expect("фикстура событий валидна");
    content.event_rules.daily_chance = daily_chance;
    Arc::new(content)
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn event(id: &str) -> EventId {
    EventId::new(id)
}

fn game_with(seed: u64, daily_chance: f64) -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, content(daily_chance)).expect("жизнь начинается")
}

/// Жизнь без случайных событий, доведённая до даты.
fn quiet_game_on(target: GameDate, seed: u64) -> Game {
    let mut game = game_with(seed, 0.0);
    advance_to(&mut game, target);
    game
}

fn advance_to(game: &mut Game, target: GameDate) {
    while game.today() < target {
        assert!(game.advance(DAY_MILLISECONDS), "время идёт");
    }
}

/// Игра у магазина «Вино-Водка» 2 июня 1985 года, событие §10.2 уже остановило время.
fn at_the_crate(seed: u64) -> Game {
    let mut game = quiet_game_on(date(1985, 6, 2), seed);
    game.start_event(&event("vodka_crate"))
        .expect("лето 1985-го, Москва, деньги есть");
    game
}

fn pending(game: &Game) -> EventView {
    game.view().pending_event.expect("событие ждёт ответа")
}

fn choose(game: &mut Game, choice: usize) {
    game.apply(Command::Choose { choice })
        .expect("выбор доступен");
}

fn level(game: &Game, faction: &str) -> i32 {
    game.reputation().level(&FactionId::new(faction))
}

fn titles_of(game: &Game) -> Vec<String> {
    game.journal()
        .iter()
        .map(|entry| game.content().events[&entry.event].title.clone())
        .collect()
}

#[test]
fn the_crate_of_section_10_2_stops_time_and_offers_three_choices() {
    let mut game = at_the_crate(7);
    let view = game.view();
    assert!(view.time.paused, "событие останавливает время");
    let shown = view.pending_event.expect("событие ждёт ответа");
    assert_eq!(shown.title, "Ящик «Столичной»");
    assert!(
        shown
            .text
            .ends_with("«Есть ящик „Столичной“, со свадьбы остался. 600 р., и я тебя не знаю»."),
        "{}",
        shown.text
    );
    assert_eq!(
        shown.choices,
        vec![
            ChoiceView {
                text: "Взять за 600 р. наличными".to_owned(),
                available: true
            },
            ChoiceView {
                text: "Отказаться".to_owned(),
                available: true
            },
            ChoiceView {
                text: "Сдать участковому".to_owned(),
                available: true
            },
        ]
    );
    assert_eq!(shown.outcome, None);
    let today = game.today();
    assert!(
        !game.advance(DAY_MILLISECONDS * 10),
        "пока событие ждёт, дни не идут"
    );
    assert_eq!(game.today(), today);
}

#[test]
fn taking_the_crate_costs_600_gives_vodka_and_triples_its_price() {
    let mut game = at_the_crate(7);
    let vodka = GoodId::new("vodka");
    let moscow = CityId::new("moscow");
    let calm_price = game
        .markets()
        .normal_kopecks(game.content(), &moscow, &vodka, game.today())
        .expect("водку продают");
    choose(&mut game, 1);
    let outcome = pending(&game).outcome.expect("выбор показывает исход");
    assert_eq!(
        outcome.text,
        "Ящик твой. Через месяц водку начнут продавать с двух часов, и цена на толкучке вырастет втрое."
    );
    assert_eq!(
        outcome.effects,
        "Торговцы +1. Деньги −600 р. Водка, ящик +1."
    );
    assert_eq!(game.view().header.money, vec!["Наличные: 400 р."]);
    assert_eq!(game.player().pocket().quantity(&vodka), 1);
    assert_eq!((level(&game, "traders"), level(&game, "militia")), (1, 0));
    let shocked_price = game
        .markets()
        .normal_kopecks(game.content(), &moscow, &vodka, game.today())
        .expect("водку продают");
    assert!(
        (shocked_price / calm_price - 3.0).abs() < 1e-9,
        "цена втрое"
    );
    assert!(game.events().flags().is_empty());
    assert!(game.events().scheduled().is_empty());
    game.apply(Command::Continue).expect("исход прочитан");
    assert_eq!(game.view().pending_event, None);
    assert!(!game.view().time.paused);
    assert!(game.advance(DAY_MILLISECONDS), "время снова идёт");
}

#[test]
fn walking_away_changes_nothing() {
    let mut game = at_the_crate(7);
    choose(&mut game, 2);
    let outcome = pending(&game).outcome.expect("выбор показывает исход");
    assert_eq!(
        outcome.text,
        "Через час мужика забирает патруль. Ничего не меняется."
    );
    assert_eq!(outcome.effects, "Ничего не изменилось.");
    assert_eq!(game.view().header.money, vec!["Наличные: 1 000 р."]);
    assert_eq!((level(&game, "traders"), level(&game, "militia")), (0, 0));
    assert!(game.events().flags().is_empty());
    assert!(!game.contacts().contains_key(&NpcId::new("vodka_man")));
}

#[test]
fn snitching_raises_the_militia_angers_traders_and_sets_the_flag() {
    let mut game = at_the_crate(7);
    choose(&mut game, 3);
    let outcome = pending(&game).outcome.expect("выбор показывает исход");
    assert_eq!(
        outcome.effects,
        "Милиция +2. Торговцы −3. Мужик в спецовке −3."
    );
    assert_eq!((level(&game, "militia"), level(&game, "traders")), (2, -3));
    assert!(
        game.events()
            .flags()
            .contains(&FlagId::new("snitched_1985"))
    );
    let man = game
        .view()
        .contacts
        .into_iter()
        .find(|contact| contact.contact == "vodka_man")
        .expect("мужик запомнил игрока и стал связью");
    assert_eq!(
        man,
        ContactView {
            contact: "vodka_man".to_owned(),
            name: "Мужик в спецовке".to_owned(),
            role: "случайный продавец".to_owned(),
            rival: false,
            relation: "обижен".to_owned(),
            debt: None,
            availability: "на связи".to_owned(),
            history: vec!["02.06.1985: Ящик «Столичной»".to_owned()],
        }
    );
}

#[test]
fn the_snitched_man_comes_back_in_half_of_the_lives() {
    let lives: Vec<(u64, bool)> = (1..=40)
        .map(|seed| {
            let mut game = at_the_crate(seed);
            choose(&mut game, 3);
            (seed, !game.events().scheduled().is_empty())
        })
        .collect();
    let returns = lives.iter().filter(|(_, comes_back)| *comes_back).count();
    assert!(
        (12..=28).contains(&returns),
        "продолжение с вероятностью 0,5 назначено в {returns} жизнях из 40"
    );
    let (seed, _) = lives
        .iter()
        .find(|(_, comes_back)| *comes_back)
        .expect("хотя бы в одной жизни мужик вернётся");
    let mut game = at_the_crate(*seed);
    choose(&mut game, 3);
    game.apply(Command::Continue).expect("исход прочитан");
    let due = date(1985, 6, 2).after_days(180);
    while game.view().pending_event.is_none() {
        assert!(game.advance(DAY_MILLISECONDS));
    }
    assert_eq!(game.today(), due, "продолжение пришло ровно через 180 дней");
    let shown = pending(&game);
    assert_eq!(shown.title, "Старый знакомый");
    assert_eq!(
        shown.text,
        "Мужик в спецовке стоит у подъезда и улыбается так, что хочется уехать в другой город."
    );
}

#[test]
fn an_event_waits_for_its_window_its_money_and_happens_once() {
    let mut early = quiet_game_on(date(1985, 5, 16), 7);
    assert_eq!(
        early.start_event(&event("vodka_crate")),
        Err(CommandError::EventNotReady {
            event: "vodka_crate".to_owned()
        }),
        "окно открывается 17 мая"
    );
    let mut broke = quiet_game_on(date(1985, 6, 2), 7);
    broke
        .apply(Command::Buy {
            good: "jeans".to_owned(),
            quantity: 3,
        })
        .expect("на джинсы денег хватает");
    assert!(
        broke.start_event(&event("vodka_crate")).is_err(),
        "после джинсов меньше 600 рублей"
    );
    let mut game = at_the_crate(7);
    choose(&mut game, 2);
    game.apply(Command::Continue).expect("исход прочитан");
    assert!(
        game.start_event(&event("vodka_crate")).is_err(),
        "однократное событие не повторяется"
    );
    assert_eq!(
        game.start_event(&event("caviar")),
        Err(CommandError::UnknownEvent {
            event: "caviar".to_owned()
        })
    );
}

#[test]
fn a_choice_needs_the_money_it_takes() {
    let mut game = at_the_crate(7);
    choose(&mut game, 3);
    game.apply(Command::Continue).expect("исход прочитан");
    game.apply(Command::Buy {
        good: "jeans".to_owned(),
        quantity: 4,
    })
    .expect("на джинсы денег хватает");
    game.start_event(&event("vodka_man_returns"))
        .expect("флаг стоит, мужик на связи");
    let availability: Vec<bool> = pending(&game)
        .choices
        .iter()
        .map(|choice| choice.available)
        .collect();
    assert_eq!(availability, vec![false, true], "откупиться не на что");
    assert_eq!(
        game.apply(Command::Choose { choice: 1 }),
        Err(CommandError::ChoiceUnavailable { choice: 1 })
    );
    choose(&mut game, 2);
}

#[test]
fn while_an_event_waits_the_world_waits() {
    let mut game = at_the_crate(7);
    assert_eq!(
        game.apply(Command::Buy {
            good: "jeans".to_owned(),
            quantity: 1
        }),
        Err(CommandError::EventPending)
    );
    assert_eq!(
        game.apply(Command::Continue),
        Err(CommandError::ChoiceRequired)
    );
    assert_eq!(
        game.apply(Command::Choose { choice: 9 }),
        Err(CommandError::NoSuchChoice {
            choice: 9,
            count: 3
        })
    );
    assert_eq!(
        game.start_event(&event("rain")),
        Err(CommandError::EventPending)
    );
    choose(&mut game, 2);
    assert_eq!(
        game.apply(Command::Choose { choice: 1 }),
        Err(CommandError::ChoiceAlreadyMade)
    );
    game.apply(Command::Continue).expect("исход прочитан");
    assert_eq!(
        game.apply(Command::Continue),
        Err(CommandError::NoPendingEvent)
    );
}

#[test]
fn the_day_draws_events_by_weight_rest_and_once() {
    let mut game = game_with(7, 1.0);
    let mut days_with_events = Vec::new();
    while game.today() < date(1985, 1, 1) {
        game.advance(DAY_MILLISECONDS);
        if game.view().pending_event.is_some() {
            days_with_events.push(game.today());
            choose(&mut game, 1);
            game.apply(Command::Continue).expect("исход прочитан");
        }
    }
    assert_eq!(
        days_with_events.len(),
        366,
        "шанс 1 — событие каждый день 1984 года"
    );
    let titles = titles_of(&game);
    let count = |title: &str| titles.iter().filter(|seen| seen.as_str() == title).count();
    let (rain, queue) = (count("Дождь"), count("Очередь"));
    assert!(
        queue >= 2 * rain && queue <= 4 * rain,
        "веса 1 и 3: дождь {rain}, очередь {queue}"
    );
    assert_eq!(count("Завоз в «Берёзку»"), 1, "однократное");
    assert_eq!(
        count("Кооператив"),
        0,
        "событие перестройки в застой не приходит"
    );
    let inspections: Vec<GameDate> = game
        .journal()
        .iter()
        .filter(|entry| entry.event == event("inspection"))
        .map(|entry| entry.date)
        .collect();
    assert!(inspections.len() >= 6, "рейдов {}", inspections.len());
    assert!(
        inspections
            .windows(2)
            .all(|pair| pair[1].day_index() - pair[0].day_index() >= 30),
        "рейд отдыхает 30 дней по правилам: {inspections:?}"
    );
    let berezka = game
        .journal()
        .iter()
        .position(|entry| entry.event == event("berezka"))
        .expect("завоз был");
    assert_eq!(game.journal()[berezka].choice, Some(0));
}

#[test]
fn an_epoch_directory_keeps_its_events_in_its_epoch() {
    let mut game = game_with(7, 1.0);
    advance_to_skipping_events(&mut game, date(1988, 2, 1));
    let perestroika = date(1988, 1, 1);
    let cooperatives: Vec<GameDate> = game
        .journal()
        .iter()
        .filter(|entry| entry.event == event("cooperative"))
        .map(|entry| entry.date)
        .collect();
    assert!(!cooperatives.is_empty(), "кооперативы — с перестройкой");
    assert!(
        cooperatives.iter().all(|day| *day >= perestroika),
        "{cooperatives:?}"
    );
    assert!(
        game.journal()
            .iter()
            .filter(|entry| entry.date >= perestroika)
            .all(|entry| entry.event == event("cooperative")),
        "события застоя кончились вместе с ним"
    );
}

/// Доводит игру до даты, отвечая на каждое событие первым доступным выбором.
fn advance_to_skipping_events(game: &mut Game, target: GameDate) {
    while game.today() < target {
        game.advance(DAY_MILLISECONDS);
        if let Some(shown) = game.view().pending_event {
            let first = shown
                .choices
                .iter()
                .position(|choice| choice.available)
                .expect("хотя бы один выбор доступен");
            choose(game, first + 1);
            game.apply(Command::Continue).expect("исход прочитан");
        }
    }
}

#[test]
fn contact_consequences_reach_the_contacts_screen() {
    let kofi_after = |choice: usize| -> (ContactView, String, Vec<String>) {
        let mut game = quiet_game_on(date(1987, 6, 1), 7);
        game.start_event(&event("kofi_holiday"))
            .expect("июнь 1987-го, Кофи на связи");
        assert_eq!(
            pending(&game).text,
            "Кофи собирается на каникулы в Аккру и просит помочь с билетом."
        );
        choose(&mut game, choice);
        let effects = pending(&game).outcome.expect("исход").effects;
        let view = game.view();
        let kofi = view
            .contacts
            .iter()
            .find(|contact| contact.contact == "kofi")
            .expect("Кофи в связях")
            .clone();
        let pocket = view
            .pocket
            .items
            .iter()
            .map(|item| format!("{} {}", item.label, item.amount))
            .collect();
        (kofi, effects, pocket)
    };

    let (kofi, effects, _) = kofi_after(1);
    assert_eq!(
        effects,
        "Деньги −50 р. Кофи: за ним услуг +1. Кофи: не на связи до 31.07.1987."
    );
    assert_eq!(kofi.debt.as_deref(), Some("За ним услуг: 1"));
    assert_eq!(kofi.availability, "не на связи до 31.07.1987");
    assert_eq!(kofi.history, vec!["01.06.1987: Кофи едет домой"]);

    let (kofi, effects, _) = kofi_after(2);
    assert_eq!(effects, "Деньги +100 р. Кофи: за тобой услуг +2.");
    assert_eq!(kofi.debt.as_deref(), Some("За тобой услуг: 2"));
    assert_eq!(kofi.relation, "знакомый");

    let (kofi, effects, pocket) = kofi_after(3);
    assert_eq!(effects, "Кофи −2. Кофи: потерян.");
    assert_eq!(
        (kofi.relation.as_str(), kofi.availability.as_str()),
        ("обижен", "потерян")
    );
    assert!(pocket.contains(&"Связей 0".to_owned()), "{pocket:?}");
}

#[test]
fn a_price_placeholder_shows_the_local_tag() {
    let mut game = quiet_game_on(date(1984, 3, 1), 7);
    game.start_event(&event("berezka"))
        .expect("завоз ещё не было");
    let jeans_buy = game.view().market.goods[0].buy.clone();
    assert_eq!(
        pending(&game).text,
        format!("В «Берёзку» завезли джинсы, на толкучке их отдают по {jeans_buy} р.")
    );
}

fn call(game: &mut Game, contact: &str) -> Result<(), CommandError> {
    game.apply(Command::Call {
        contact: contact.to_owned(),
    })
}

#[test]
fn a_call_starts_a_ready_event_about_the_contact() {
    let mut game = quiet_game_on(date(1987, 6, 10), 1);
    call(&mut game, "kofi").expect("Кофи на связи");
    assert_eq!(pending(&game).title, "Кофи едет домой");
}

#[test]
fn a_contact_with_nothing_to_say_does_not_answer() {
    let mut game = quiet_game_on(date(1985, 3, 1), 1);
    call(&mut game, "kofi").expect("звонок проходит");
    assert!(game.view().pending_event.is_none());
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Кофи не берёт трубку")
    );
}

#[test]
fn only_a_reachable_contact_can_be_called() {
    let mut game = quiet_game_on(date(1987, 6, 10), 1);
    assert_eq!(
        call(&mut game, "vodka_man"),
        Err(CommandError::NotAContact {
            contact: "vodka_man".to_owned()
        })
    );
    call(&mut game, "kofi").expect("Кофи на связи");
    choose(&mut game, 1);
    game.apply(Command::Continue).expect("исход прочитан");
    assert_eq!(
        call(&mut game, "kofi"),
        Err(CommandError::ContactAway {
            name: "Кофи".to_owned(),
            until: "09.08.1987".to_owned()
        }),
        "уехал на каникулы на 60 дней"
    );
    let mut quarrel = quiet_game_on(date(1987, 6, 10), 1);
    call(&mut quarrel, "kofi").expect("Кофи на связи");
    choose(&mut quarrel, 3);
    quarrel.apply(Command::Continue).expect("исход прочитан");
    assert_eq!(
        call(&mut quarrel, "kofi"),
        Err(CommandError::ContactLost {
            name: "Кофи".to_owned()
        })
    );
}
