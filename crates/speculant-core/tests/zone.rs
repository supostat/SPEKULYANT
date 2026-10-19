//! Зона (§12, решение 15.8): приговор снимает игрока с улицы, неделя зоны идёт за время одного
//! дня на воле, промзона платит сигаретами, передачи и услуги меняют деньги на сигареты и чай,
//! безделье растит авторитет, события зоны приходят только в зоне, доверенные лица ведут дело
//! честно, воруют или исчезают, а выход — по сроку, по УДО, по амнистии и за взятку — ставит флаг
//! «Бывшего сидельца» и оборачивает авторитет связями «оттуда».

use std::path::Path;
use std::sync::Arc;

use speculant_core::business::{BusinessId, MonthReport};
use speculant_core::content::{
    Content, ContentError, ContentFile, EventId, FactionId, FlagId, NpcId,
};
use speculant_core::economy::{Currency, Real};
use speculant_core::time::{FastForward, GameDate};
use speculant_core::view::{
    LevelView, PocketItemView, Screen, ZoneOfferView, ZoneServiceView, ZoneView,
};
use speculant_core::{Command, CommandError, Game, LifeSettings};

const ZONE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/zone");
const DAY_MILLISECONDS: u32 = 1800;
const STALL: &str = "stall@moscow";
const SOVIET_CAMP: &str = "ИТК общего режима, Мордовия";
const WILD_CAMP: &str = "Колония общего режима, Тверская область";

fn load() -> Content {
    Content::load(Path::new(ZONE)).expect("фикстура зоны валидна")
}

fn settings(biography: &str) -> LifeSettings {
    LifeSettings {
        biography: biography.to_owned(),
        ironman: true,
        seed: 7,
    }
}

fn game_with(content: Content) -> Game {
    Game::new_life(&settings("student"), Arc::new(content)).expect("жизнь начинается")
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn play(game: &mut Game, command: Command) {
    let shown = format!("{command:?}");
    game.apply(command)
        .unwrap_or_else(|error| panic!("{shown}: {error}"));
}

/// Событие от начала до выбора `choice`; исход ещё не прочитан. Возвращает строку перемен.
fn choose(game: &mut Game, event: &str, choice: usize) -> String {
    game.start_event(&EventId::new(event))
        .unwrap_or_else(|error| panic!("{event}: {error}"));
    play(game, Command::Choose { choice });
    game.view()
        .pending_event
        .and_then(|event| event.outcome)
        .expect("исход выбора")
        .effects
}

/// Суд даёт год: признание в событии ареста, исход прочитан.
fn sentence_to_a_year(game: &mut Game) {
    choose(game, "arrest", 1);
    play(game, Command::Continue);
}

/// Живёт до даты или до конца жизни: на воле — по дню, в зоне — по неделе за раз.
fn live_until(game: &mut Game, until: GameDate) {
    while game.today() < until && game.advance(DAY_MILLISECONDS) {}
}

/// Сидит, пока не выйдет, на перемотке.
fn serve_out(game: &mut Game) {
    play(
        game,
        Command::FastForward {
            until: FastForward::UntilEvent,
        },
    );
    while game.zone().is_some() && game.summary().is_none() {
        assert!(game.advance(DAY_MILLISECONDS), "время в зоне идёт");
        assert!(
            game.events().pending().is_none(),
            "в тихой зоне событий нет"
        );
    }
}

fn zone_view(game: &Game) -> ZoneView {
    game.view().zone.expect("игрок в зоне")
}

fn stash(game: &Game) -> Vec<(String, String)> {
    zone_view(game)
        .stash
        .into_iter()
        .map(|PocketItemView { label, amount }| (label, amount))
        .collect()
}

fn packs(cigarettes: &str, tea: &str) -> Vec<(String, String)> {
    vec![
        ("Сигареты".to_owned(), cigarettes.to_owned()),
        ("Чай".to_owned(), tea.to_owned()),
    ]
}

fn authority(game: &Game) -> String {
    zone_view(game).authority.label
}

fn messages(game: &Game) -> Vec<String> {
    game.view().messages
}

fn cash_kopecks(game: &Game) -> i64 {
    game.player().money().cash().minor_units()
}

/// Палатка у метро с Геной Кабановым: лояльность 2, честность 3.
fn stall_with_gena(game: &mut Game) {
    play(
        game,
        Command::OpenBusiness {
            kind: "stall".to_owned(),
        },
    );
    play(
        game,
        Command::Hire {
            business: STALL.to_owned(),
            candidate: "gena_kabanov".to_owned(),
        },
    );
}

fn stall_report(game: &Game) -> MonthReport {
    let id = BusinessId::parse(STALL).expect("идентификатор дела");
    *game.businesses()[&id]
        .last_report()
        .expect("отчёт за прошлый месяц")
}

#[test]
fn a_sentence_takes_the_player_off_the_street_into_the_zone() {
    let mut game = game_with(load());
    assert_eq!(game.view().zone, None);
    let effects = choose(&mut game, "arrest", 1);
    assert_eq!(effects, "Срок: 1 год.");
    assert_eq!(
        game.view().screen,
        Screen::Market,
        "экран зоны — когда исход прочитан"
    );
    play(&mut game, Command::Continue);

    let view = game.view();
    assert_eq!(view.screen, Screen::Zone);
    assert_eq!(view.header.city, "В зоне");
    assert_eq!(view.market.place, SOVIET_CAMP);
    assert!(view.market.goods.is_empty(), "в зоне толкучки нет");
    assert!(view.travel.destinations.is_empty(), "из зоны не ездят");
    assert!(view.business_kinds.is_empty() && view.candidates.is_empty());
    assert!(view.exits.iter().all(|exit| !exit.available));
    assert_eq!(
        view.zone,
        Some(ZoneView {
            camp: SOVIET_CAMP.to_owned(),
            sentence: "1 год".to_owned(),
            release: "01.01.1985".to_owned(),
            authority: LevelView {
                label: "первоход".to_owned(),
                level: 1,
                scale: 5,
            },
            stash: vec![
                PocketItemView {
                    label: "Сигареты".to_owned(),
                    amount: "0 пачек".to_owned(),
                },
                PocketItemView {
                    label: "Чай".to_owned(),
                    amount: "0 пачек".to_owned(),
                },
            ],
            working: false,
            services: vec![
                ZoneServiceView {
                    service: "common_fund".to_owned(),
                    title: "На общак".to_owned(),
                    price: "10 пачек сигарет, 5 пачек чая".to_owned(),
                    available: false,
                },
                ZoneServiceView {
                    service: "infirmary".to_owned(),
                    title: "Больничка".to_owned(),
                    price: "10 пачек сигарет".to_owned(),
                    available: false,
                },
            ],
            parcel: ZoneOfferView {
                terms: "30 р.".to_owned(),
                available: true,
            },
            parole: ZoneOfferView {
                terms: "с 02.07.1984".to_owned(),
                available: false,
            },
            bribe: None,
            trustees: Vec::new(),
        })
    );
    assert!(
        messages(&game).contains(&format!("Осуждён: 1 год, {SOVIET_CAMP}")),
        "{:?}",
        messages(&game)
    );
}

#[test]
fn a_week_of_the_zone_passes_in_the_time_of_a_day_of_freedom() {
    let mut game = game_with(load());
    assert!(game.advance(DAY_MILLISECONDS));
    assert_eq!(game.today(), date(1984, 1, 2), "на воле — день");
    sentence_to_a_year(&mut game);
    assert!(game.advance(DAY_MILLISECONDS));
    assert_eq!(game.today(), date(1984, 1, 9), "в зоне — неделя");
    serve_out(&mut game);
    assert_eq!(game.today(), date(1985, 1, 2), "вышел в день конца срока");
    assert!(game.advance(DAY_MILLISECONDS));
    assert_eq!(game.today(), date(1985, 1, 3), "на воле снова день");
}

#[test]
fn a_sentence_beyond_the_limit_ends_the_life_with_a_verdict() {
    let mut game = game_with(load());
    let effects = choose(&mut game, "arrest", 2);
    assert_eq!(effects, "Срок: 12 лет 6 месяцев.");
    play(&mut game, Command::Continue);
    assert_eq!(game.zone(), None, "в зону не попадает — жизнь кончилась");
    let summary = game.summary().expect("итоги жизни");
    assert_eq!(summary.ending.as_str(), "prison");
    assert_eq!(game.view().screen, Screen::Summary);
}

#[test]
fn the_promzona_pays_in_cigarettes_and_idleness_raises_authority() {
    let mut content = load();
    content.zone.idle_authority_chance = 1.0;
    let mut game = game_with(content);
    sentence_to_a_year(&mut game);
    play(&mut game, Command::SetWork { work: true });
    assert!(zone_view(&game).working);
    for _ in 0..3 {
        assert!(game.advance(DAY_MILLISECONDS));
    }
    assert_eq!(stash(&game), packs("12 пачек", "0 пачек"));
    assert_eq!(
        authority(&game),
        "первоход",
        "работающий в авторитете не растёт"
    );

    play(&mut game, Command::SetWork { work: false });
    let mut ladder = Vec::new();
    for _ in 0..6 {
        assert!(game.advance(DAY_MILLISECONDS));
        ladder.push(authority(&game));
    }
    assert_eq!(
        ladder,
        [
            "мужик",
            "уважаемый",
            "блатной",
            "авторитет",
            "авторитет",
            "авторитет"
        ],
        "выше вершины авторитет не растёт"
    );
    assert_eq!(
        stash(&game),
        packs("12 пачек", "0 пачек"),
        "без работы — без сигарет"
    );
    assert!(messages(&game).contains(&"Авторитет: уважаемый".to_owned()));
}

#[test]
fn parcels_from_freedom_and_services_trade_money_for_cigarettes_and_tea() {
    let mut game = game_with(load());
    sentence_to_a_year(&mut game);
    let cash = cash_kopecks(&game);
    play(&mut game, Command::OrderParcel);
    assert_eq!(
        cash_kopecks(&game),
        cash - 3_000,
        "передача — 30 р. из наличных"
    );
    assert_eq!(stash(&game), packs("20 пачек", "5 пачек"));
    assert_eq!(
        game.apply(Command::OrderParcel),
        Err(CommandError::ParcelTooSoon {
            from: "01.04.1984".to_owned()
        }),
        "советская зона разрешает передачу раз в 13 недель"
    );
    assert_eq!(
        zone_view(&game).parcel,
        ZoneOfferView {
            terms: "с 01.04.1984".to_owned(),
            available: false,
        }
    );

    play(
        &mut game,
        Command::UseService {
            service: "common_fund".to_owned(),
        },
    );
    assert_eq!(stash(&game), packs("10 пачек", "0 пачек"));
    assert_eq!(authority(&game), "мужик");
    assert!(messages(&game).contains(&"На общак за 10 пачек сигарет, 5 пачек чая".to_owned()));
    assert_eq!(
        game.apply(Command::UseService {
            service: "common_fund".to_owned()
        }),
        Err(CommandError::StashTooSmall {
            title: "На общак".to_owned(),
            needed: "10 пачек сигарет, 5 пачек чая".to_owned(),
            stash: "10 пачек сигарет".to_owned(),
        })
    );
    play(
        &mut game,
        Command::UseService {
            service: "infirmary".to_owned(),
        },
    );
    assert_eq!(stash(&game), packs("0 пачек", "0 пачек"));
    assert_eq!(
        game.character().health(),
        100,
        "больничка лечит до предела шкалы"
    );
    assert_eq!(
        game.apply(Command::UseService {
            service: "massage".to_owned()
        }),
        Err(CommandError::UnknownService {
            service: "massage".to_owned()
        })
    );
}

#[test]
fn zone_events_come_only_in_the_zone_and_move_the_stash_the_authority_and_the_term() {
    let mut game = game_with(load());
    assert_eq!(
        game.start_event(&EventId::new("card_game")),
        Err(CommandError::EventNotReady {
            event: "card_game".to_owned()
        }),
        "на воле событий зоны нет"
    );
    sentence_to_a_year(&mut game);
    assert_eq!(
        game.start_event(&EventId::new("heart_attack")),
        Err(CommandError::EventNotReady {
            event: "heart_attack".to_owned()
        }),
        "в зоне событий воли нет"
    );
    assert_eq!(
        game.start_event(&EventId::new("elder_word")),
        Err(CommandError::EventNotReady {
            event: "elder_word".to_owned()
        }),
        "смотрящий не зовёт первохода"
    );

    game.start_event(&EventId::new("card_game"))
        .expect("в зоне событие зоны готово");
    let choices: Vec<bool> = game
        .view()
        .pending_event
        .expect("событие")
        .choices
        .iter()
        .map(|choice| choice.available)
        .collect();
    assert_eq!(choices, [true, false], "пачку чая не поставить без чая");
    assert_eq!(
        game.apply(Command::Choose { choice: 2 }),
        Err(CommandError::ChoiceUnavailable { choice: 2 })
    );
    play(&mut game, Command::Choose { choice: 1 });
    let effects = game
        .view()
        .pending_event
        .and_then(|event| event.outcome)
        .expect("исход")
        .effects;
    assert_eq!(effects, "Сигареты +10. Авторитет: мужик.");
    play(&mut game, Command::Continue);
    assert_eq!(game.view().screen, Screen::Zone);

    let effects = choose(&mut game, "canteen_fight", 1);
    assert_eq!(
        effects,
        "Здоровье: здоров. Авторитет: уважаемый. Выход по сроку — 01.07.1985."
    );
    play(&mut game, Command::Continue);
    let view = zone_view(&game);
    assert_eq!(
        (view.sentence.as_str(), view.release.as_str()),
        ("1 год 6 месяцев", "01.07.1985")
    );
    assert_eq!(
        game.start_event(&EventId::new("canteen_fight")),
        Err(CommandError::EventNotReady {
            event: "canteen_fight".to_owned()
        }),
        "уважаемого первоходом не задирают"
    );
    let effects = choose(&mut game, "elder_word", 1);
    assert_eq!(effects, "Авторитет: блатной.");
}

#[test]
fn the_zone_draws_its_own_events_by_the_week() {
    let mut content = load();
    content.zone.event_chance = 1.0;
    let mut game = game_with(content);
    sentence_to_a_year(&mut game);
    assert!(game.advance(DAY_MILLISECONDS));
    let pending = game
        .events()
        .pending()
        .expect("событие зоны в конце недели");
    assert_eq!(game.today(), date(1984, 1, 8), "событие остановило неделю");
    assert_eq!(
        game.content().events[&pending.event].epoch,
        None,
        "в зоне приходят события зоны"
    );
}

#[test]
fn a_term_grown_past_the_limit_ends_the_life_in_the_zone() {
    let mut content = load();
    content.zone.longest_sentence_months = 15;
    let mut game = game_with(content);
    sentence_to_a_year(&mut game);
    choose(&mut game, "canteen_fight", 1);
    play(&mut game, Command::Continue);
    assert_eq!(
        game.summary().expect("итоги").ending.as_str(),
        "prison",
        "полтора года больше предела в пятнадцать месяцев"
    );
}

#[test]
fn a_case_from_freedom_adds_to_the_term() {
    let mut content = load();
    for level in &mut content.business.risk_levels {
        level.monthly_chance = 1.0;
    }
    for form in content.business_forms.values_mut() {
        form.racket_share = 0.0;
    }
    let mut game = game_with(content);
    stall_with_gena(&mut game);
    sentence_to_a_year(&mut game);
    assert!(game.advance(DAY_MILLISECONDS));
    let pending = game
        .events()
        .pending()
        .expect("проверка зовёт хозяина и в зону");
    assert_eq!(pending.event.as_str(), "inspection_case");
    play(&mut game, Command::Choose { choice: 1 });
    play(&mut game, Command::Continue);
    let view = zone_view(&game);
    assert_eq!(
        (view.sentence.as_str(), view.release.as_str()),
        ("1 год 6 месяцев", "01.07.1985")
    );
    assert!(
        messages(&game)
            .contains(&"К сроку добавили 6 месяцев. Выход по сроку — 01.07.1985".to_owned()),
        "{:?}",
        messages(&game)
    );
}

/// Деления черт Гены Кабанова в прогнозе: политики фикстуры их не сдвигают.
const GENA_LOYALTY: usize = 2;
const GENA_HONESTY: usize = 3;

/// Шансы по делениям черты: наверняка на одном делении, никогда — на остальных.
fn certain_at(level: usize) -> Vec<f64> {
    let mut chances = vec![0.0; 5];
    chances[level - 1] = 1.0;
    chances
}

/// Палатка с Геной, год зоны с первого дня и расчёт января первого февраля.
fn january_under_a_trustee(vanish: Vec<f64>, steal: Vec<f64>, in_zone: bool) -> Game {
    let mut content = load();
    content.zone.trustees.vanish_chances = vanish;
    content.zone.trustees.steal_chances = steal;
    let mut game = game_with(content);
    stall_with_gena(&mut game);
    if in_zone {
        sentence_to_a_year(&mut game);
    }
    live_until(&mut game, date(1984, 2, 1));
    game
}

#[test]
fn trustees_run_the_business_honestly_steal_or_vanish_by_their_traits() {
    let honest = january_under_a_trustee(certain_at(GENA_HONESTY), certain_at(GENA_LOYALTY), true);
    let report = stall_report(&honest);
    assert!(report.income.minor_units() > 0);
    assert_eq!(
        report.stolen.minor_units(),
        0,
        "лояльность решает, исчезнет ли, честность — ворует ли; у Гены обе спокойны"
    );
    assert_eq!(
        zone_view(&honest).trustees,
        ["Палатка у метро, Москва — Гена Кабанов"]
    );

    let thief = january_under_a_trustee(certain_at(GENA_HONESTY), certain_at(GENA_HONESTY), true);
    let report = stall_report(&thief);
    assert!(
        (report.stolen.minor_units() * 2 - report.income.minor_units()).abs() <= 1,
        "ворующий уносит половину выручки: {report:?}"
    );

    let vanished = january_under_a_trustee(certain_at(GENA_LOYALTY), vec![0.0; 5], true);
    let report = stall_report(&vanished);
    assert_eq!(
        report.stolen, report.income,
        "исчезнувший уносит всю выручку"
    );
    assert_eq!(
        zone_view(&vanished).trustees,
        ["Палатка у метро, Москва — никого"]
    );
    assert!(
        messages(&vanished)
            .contains(&"Палатка у метро, Москва: Гена Кабанов исчез с выручкой".to_owned())
    );
    let gena = &vanished.contacts()[&NpcId::new("gena_kabanov")];
    assert!(gena.is_lost(), "исчезнувшего не найти");

    let watched =
        january_under_a_trustee(certain_at(GENA_LOYALTY), certain_at(GENA_HONESTY), false);
    let id = BusinessId::parse(STALL).expect("идентификатор дела");
    assert!(
        watched.businesses()[&id].manager().is_some(),
        "при хозяине на воле управляющий не исчезает"
    );
    assert_eq!(stall_report(&watched).stolen.minor_units(), 0);
}

#[test]
fn release_by_term_sets_the_flag_and_turns_authority_into_ties_from_there() {
    let mut game = game_with(load());
    sentence_to_a_year(&mut game);
    play(&mut game, Command::OrderParcel);
    play(
        &mut game,
        Command::UseService {
            service: "common_fund".to_owned(),
        },
    );
    choose(&mut game, "card_game", 1);
    play(&mut game, Command::Continue);
    assert_eq!(authority(&game), "уважаемый");

    serve_out(&mut game);
    let view = game.view();
    assert_eq!(view.zone, None);
    assert_eq!(view.screen, Screen::Market);
    assert_eq!(view.header.city, "Москва");
    assert_eq!(game.fast_forward(), None, "выход останавливает перемотку");
    assert!(
        game.events()
            .flags()
            .contains(&FlagId::new("released_from_zone"))
    );
    assert_eq!(game.reputation().level(&FactionId::new("bandits")), 1);
    let messages = messages(&game);
    assert!(messages.contains(&"Вышел на волю по сроку".to_owned()));
    assert!(messages.contains(&"Связи «оттуда»: Бандиты +1".to_owned()));
    assert_eq!(
        game.apply(Command::OrderParcel),
        Err(CommandError::NotInZone)
    );
}

#[test]
fn parole_needs_half_the_term_a_work_record_and_modest_authority() {
    let mut idler = game_with(load());
    sentence_to_a_year(&mut idler);
    assert_eq!(
        idler.apply(Command::ApplyForParole),
        Err(CommandError::ParoleTooEarly {
            from: "02.07.1984".to_owned()
        })
    );
    live_until(&mut idler, date(1984, 7, 2));
    play(&mut idler, Command::ApplyForParole);
    assert!(
        idler.zone().is_some(),
        "без трудовой характеристики УДО не дают"
    );
    assert!(
        messages(&idler)
            .iter()
            .any(|message| message.starts_with("УДО: отказ, следующая просьба — с")),
        "{:?}",
        messages(&idler)
    );
    assert!(matches!(
        idler.apply(Command::ApplyForParole),
        Err(CommandError::ParoleTooEarly { .. })
    ));

    let mut worker = game_with(load());
    sentence_to_a_year(&mut worker);
    play(&mut worker, Command::SetWork { work: true });
    live_until(&mut worker, date(1984, 7, 2));
    assert!(zone_view(&worker).parole.available);
    play(&mut worker, Command::ApplyForParole);
    assert_eq!(worker.zone(), None, "работавшему — УДО");
    assert!(messages(&worker).contains(&"Вышел на волю по УДО".to_owned()));
    assert_eq!(worker.view().screen, Screen::Market);

    let mut content = load();
    content.zone.start_authority = 4;
    let mut thief_in_law = game_with(content);
    sentence_to_a_year(&mut thief_in_law);
    play(&mut thief_in_law, Command::SetWork { work: true });
    live_until(&mut thief_in_law, date(1984, 7, 2));
    play(&mut thief_in_law, Command::ApplyForParole);
    assert!(
        thief_in_law.zone().is_some(),
        "блатному УДО не дают и за работу"
    );
}

#[test]
fn an_amnesty_is_a_zone_event_inside_its_window() {
    let mut game = game_with(load());
    live_until(&mut game, date(1987, 6, 10));
    sentence_to_a_year(&mut game);
    assert_eq!(
        game.start_event(&EventId::new("amnesty_1987")),
        Err(CommandError::EventNotReady {
            event: "amnesty_1987".to_owned()
        }),
        "указ ещё не вышел"
    );
    live_until(&mut game, date(1987, 6, 18));
    let effects = choose(&mut game, "amnesty_1987", 1);
    assert_eq!(effects, "Вышел на волю по амнистии.");
    assert_eq!(
        game.view().screen,
        Screen::Zone,
        "рынок — когда исход прочитан"
    );
    play(&mut game, Command::Continue);
    assert_eq!(game.zone(), None);
    assert_eq!(game.view().screen, Screen::Market);
}

#[test]
fn the_soviet_zone_sells_no_freedom_and_the_zone_of_the_nineties_does() {
    let mut soviet = game_with(load());
    sentence_to_a_year(&mut soviet);
    assert_eq!(
        soviet.apply(Command::BribeForRelease),
        Err(CommandError::NoBribe {
            camp: SOVIET_CAMP.to_owned()
        })
    );

    let mut game = game_with(load());
    live_until(&mut game, date(1991, 12, 20));
    sentence_to_a_year(&mut game);
    assert_eq!(zone_view(&game).bribe, None);
    live_until(&mut game, date(1992, 1, 1));
    let view = zone_view(&game);
    assert_eq!(view.camp, WILD_CAMP, "зона сменилась с эпохой");
    let bribe = view.bribe.expect("зона девяностых продаёт волю");
    assert!(bribe.available);
    let today = game.today();
    let price = game
        .content()
        .money
        .from_real(Real::from_kopecks(300_000), Currency::Ruble, today);
    assert_eq!(bribe.terms, game.content().money.format_money(price, today));
    let cash = cash_kopecks(&game);
    play(&mut game, Command::BribeForRelease);
    assert_eq!(cash_kopecks(&game), cash - price.minor_units());
    assert_eq!(game.zone(), None);
    assert!(messages(&game).contains(&"Вышел на волю за взятку".to_owned()));
}

#[test]
fn the_ex_convict_opens_for_the_next_life_after_a_release() {
    let mut game = game_with(load());
    assert_eq!(
        game.apply(Command::NewLife(settings("ex_convict"))),
        Err(CommandError::BiographyLocked {
            biography: "ex_convict".to_owned(),
            title: "Бывший сиделец".to_owned(),
            hint: "Открывается после выхода с зоны".to_owned(),
        })
    );
    sentence_to_a_year(&mut game);
    serve_out(&mut game);
    choose(&mut game, "heart_attack", 1);
    play(&mut game, Command::Continue);
    assert_eq!(
        game.view().summary.expect("итоги").discoveries,
        ["Открыто: биография «Бывший сиделец»"]
    );
    play(&mut game, Command::NewLife(settings("ex_convict")));
    assert_eq!(game.character().biography().as_str(), "ex_convict");
}

#[test]
fn the_zone_wears_health_down_to_death() {
    let mut content = load();
    content.zone.health_loss_chance = 1.0;
    let mut game = game_with(content);
    sentence_to_a_year(&mut game);
    assert!(game.advance(DAY_MILLISECONDS));
    assert_eq!(game.character().health(), 40);
    assert!(game.advance(DAY_MILLISECONDS));
    assert_eq!(
        game.summary().expect("итоги").ending.as_str(),
        "death_health"
    );
}

#[test]
fn freedom_commands_wait_for_the_release_and_zone_commands_for_the_zone() {
    let mut game = game_with(load());
    for command in [
        Command::SetWork { work: true },
        Command::UseService {
            service: "infirmary".to_owned(),
        },
        Command::OrderParcel,
        Command::ApplyForParole,
        Command::BribeForRelease,
    ] {
        assert_eq!(
            game.apply(command.clone()),
            Err(CommandError::NotInZone),
            "{command:?}"
        );
    }
    stall_with_gena(&mut game);
    sentence_to_a_year(&mut game);
    for command in [
        Command::Buy {
            good: "jeans".to_owned(),
            quantity: 1,
        },
        Command::Sell {
            good: "jeans".to_owned(),
            quantity: 1,
        },
        Command::Travel {
            destination: "leningrad".to_owned(),
        },
        Command::Deposit {
            place: "mattress".to_owned(),
            amount: 1,
        },
        Command::Withdraw {
            place: "mattress".to_owned(),
            amount: 1,
        },
        Command::OpenBusiness {
            kind: "kiosks".to_owned(),
        },
        Command::SetPolicy {
            business: STALL.to_owned(),
            policy: "roof".to_owned(),
            option: 1,
        },
        Command::Fire {
            business: STALL.to_owned(),
            employee: "gena_kabanov".to_owned(),
        },
        Command::Call {
            contact: "gena_kabanov".to_owned(),
        },
        Command::BuyHousing {
            housing: "dorm_bed".to_owned(),
        },
        Command::Retire {
            ending: "prison".to_owned(),
        },
    ] {
        assert_eq!(
            game.apply(command.clone()),
            Err(CommandError::InZone),
            "{command:?}"
        );
    }
    game.start_event(&EventId::new("card_game"))
        .expect("событие зоны");
    assert_eq!(
        game.apply(Command::OrderParcel),
        Err(CommandError::EventPending),
        "пока событие ждёт ответа, передачу не заказать"
    );
}

/// Файлы фикстуры с правкой одного файла.
fn with_file(path: &str, edit: impl Fn(&str) -> String) -> Vec<ContentFile> {
    let mut files = Content::read_files(Path::new(ZONE)).expect("файлы фикстуры читаются");
    let file = files
        .iter_mut()
        .find(|file| file.path == path)
        .unwrap_or_else(|| panic!("файла {path} нет"));
    file.text = edit(&file.text);
    files
}

fn errors_of(files: Vec<ContentFile>) -> Vec<String> {
    Content::from_files(files)
        .expect_err("набор отвергнут")
        .iter()
        .map(ContentError::to_string)
        .collect()
}

/// Порча одного файла фикстуры и ошибка, которой загрузка на неё отвечает.
struct Defect {
    path: &'static str,
    edit: fn(&str) -> String,
    error: &'static str,
}

#[test]
fn the_loader_keeps_the_zone_and_freedom_apart() {
    let defects = [
        Defect {
            path: "zone/rules.toml",
            edit: |text| text.replace("life_ending = \"prison\"", "life_ending = \"death_health\""),
            error: "zone/rules.toml:6: финал срока death_health — не зона",
        },
        Defect {
            path: "zone/rules.toml",
            edit: |text| text.replace("ties = [0, 0, 1, 2, 3]", "ties = [0, 0, 1]"),
            error: "zone/rules.toml:17: связей «оттуда» 3 значений, а уровней авторитета 5",
        },
        Defect {
            path: "zone/rules.toml",
            edit: |text| {
                text.split("[camps.wild_market]")
                    .next()
                    .expect("начало файла")
                    .to_owned()
            },
            error: "zone/rules.toml:28: нет зоны для эпохи wild_market",
        },
        Defect {
            path: "events/stagnation/arrest.toml",
            edit: |text| text.replace("sentence_months = 12", "zone = { cigarettes = 5 }"),
            error: "events/stagnation/arrest.toml:5: выбор 1: последствие zone — только у событий зоны в zone/events/",
        },
        Defect {
            path: "zone/events/card_game.toml",
            edit: |text| {
                text.replace(
                    "zone = { cigarettes = 10, authority = 1 }",
                    "sentence_months = 6",
                )
            },
            error: "zone/events/card_game.toml:5: выбор 1: в зоне срок прибавляет zone.term_months, а не sentence_months",
        },
        Defect {
            path: "events/stagnation/heart_attack.toml",
            edit: |text| format!("{text}\n[conditions]\nauthority = {{ at_least = 2 }}\n"),
            error: "events/stagnation/heart_attack.toml:10: условие authority — только у событий зоны в zone/events/",
        },
        Defect {
            path: "zone/services/infirmary.toml",
            edit: |text| text.replace("price = { cigarettes = 10 }", "price = {}"),
            error: "zone/services/infirmary.toml:2: услуга бесплатной не бывает: цена в сигаретах или чае",
        },
    ];
    for Defect { path, edit, error } in defects {
        let errors = errors_of(with_file(path, edit));
        assert!(
            errors.iter().any(|reported| reported == error),
            "{path}: ждали «{error}», а пришло {errors:?}"
        );
    }
}
