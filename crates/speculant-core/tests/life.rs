//! Жизнь (§11, решения 15.4 и 15.5): старт по биографии с именем и возрастом по сиду, закрытые
//! биографии и их открытие, старение и здоровье, жильё и деньги под матрасом, финалы — от
//! событий, от здоровья и по своей воле — и итоги в духе эпохи.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{Content, EventId, GoodId, HousingId};
use speculant_core::life::PagerDevice;
use speculant_core::time::GameDate;
use speculant_core::view::{Screen, StateView, SummaryLineView};
use speculant_core::{Command, CommandError, Game, LifeSettings};

const LIFE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/life");
const DAY_MILLISECONDS: u32 = 1800;

fn load() -> Content {
    Content::load(Path::new(LIFE)).expect("фикстура жизни валидна")
}

fn settings(biography: &str, seed: u64) -> LifeSettings {
    LifeSettings {
        biography: biography.to_owned(),
        ironman: true,
        seed,
    }
}

fn game_with(content: Content, seed: u64) -> Game {
    Game::new_life(&settings("student", seed), Arc::new(content)).expect("жизнь начинается")
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn play(game: &mut Game, command: Command) {
    let shown = format!("{command:?}");
    game.apply(command)
        .unwrap_or_else(|error| panic!("{shown}: {error}"));
}

/// Событие с одним выбором — от начала до прочитанного исхода.
fn live_through(game: &mut Game, event: &str) {
    game.start_event(&EventId::new(event))
        .unwrap_or_else(|error| panic!("{event}: {error}"));
    play(game, Command::Choose { choice: 1 });
    play(game, Command::Continue);
}

/// Живёт по дню до даты или до конца жизни.
fn live_until(game: &mut Game, until: GameDate) {
    while game.today() < until && game.advance(DAY_MILLISECONDS) {}
}

fn summary_lines(view: &StateView) -> Vec<(String, String)> {
    view.summary
        .as_ref()
        .expect("итоги жизни")
        .lines
        .iter()
        .map(|SummaryLineView { label, value }| (label.clone(), value.clone()))
        .collect()
}

#[test]
fn a_life_starts_from_its_biography_with_a_name_and_an_age_by_seed() {
    let content = load();
    let mut names = BTreeSet::new();
    let mut ages = BTreeSet::new();
    for seed in 1..=40 {
        let game = game_with(content.clone(), seed);
        let character = game.character();
        let age = character.age_on(GameDate::START);
        assert!((20..=23).contains(&age), "сид {seed}: {age} лет");
        assert!(character.born() < GameDate::START);
        assert_eq!(
            character.name().split(' ').count(),
            3,
            "фамилия, имя, отчество"
        );
        assert_eq!(
            character.name(),
            game_with(content.clone(), seed).character().name(),
            "один сид — одно имя"
        );
        names.insert(character.name().to_owned());
        ages.insert(age);
    }
    assert!(names.len() >= 20, "имена разные: {}", names.len());
    assert_eq!(
        ages.len(),
        4,
        "все возрасты биографии встречаются: {ages:?}"
    );

    let view = game_with(content, 7).view();
    assert_eq!(view.header.money, ["Наличные: 5 000 р."]);
    assert_eq!(view.character.health, "здоров");
    assert_eq!(view.character.housing, "Койка в общежитии");
    assert!(
        ["20 лет", "21 год", "22 года", "23 года"].contains(&view.character.age.as_str()),
        "{}",
        view.character.age
    );
    assert_eq!(view.summary, None);
}

#[test]
fn a_locked_biography_opens_after_the_life_that_earned_it() {
    let mut game = game_with(load(), 7);
    let biographies: Vec<(String, bool, Option<String>)> = game
        .view()
        .biographies
        .into_iter()
        .map(|biography| {
            (
                biography.biography,
                biography.unlocked,
                biography.unlock_hint,
            )
        })
        .collect();
    assert_eq!(
        biographies,
        [
            (
                "party_son".to_owned(),
                false,
                Some("Открывается после выхода на номенклатуру в одной из жизней".to_owned())
            ),
            ("student".to_owned(), true, None),
        ]
    );
    assert_eq!(
        game.apply(Command::NewLife(settings("party_son", 7))),
        Err(CommandError::BiographyLocked {
            biography: "party_son".to_owned(),
            title: "Сын партработника".to_owned(),
            hint: "Открывается после выхода на номенклатуру в одной из жизней".to_owned()
        })
    );

    live_through(&mut game, "nomenklatura_offer");
    play(
        &mut game,
        Command::Retire {
            ending: "rest".to_owned(),
        },
    );
    let view = game.view();
    assert_eq!(
        view.summary.expect("итоги").discoveries,
        ["Открыто: биография «Сын партработника»"]
    );
    assert!(view.biographies.iter().all(|biography| biography.unlocked));

    play(&mut game, Command::NewLife(settings("party_son", 8)));
    let view = game.view();
    assert_eq!(view.header.money, ["Наличные: 20 000 р."]);
    assert_eq!(view.character.housing, "Кооперативная квартира");
    assert_eq!(game.today(), GameDate::START);

    live_through(&mut game, "nomenklatura_offer");
    play(
        &mut game,
        Command::Retire {
            ending: "rest".to_owned(),
        },
    );
    assert!(
        game.view().summary.expect("итоги").discoveries.is_empty(),
        "открытое однажды второй раз не открывается"
    );
    assert_eq!(
        Game::new_life(&settings("party_son", 7), Arc::new(load())).err(),
        Some(CommandError::BiographyLocked {
            biography: "party_son".to_owned(),
            title: "Сын партработника".to_owned(),
            hint: "Открывается после выхода на номенклатуру в одной из жизней".to_owned()
        }),
        "открытия живут в сеансе; на диск их сохранит метапрогрессия"
    );
}

#[test]
fn years_wear_down_health_until_the_heart_gives_out() {
    let mut aging = load();
    aging.life.aging_from = 20;
    aging.life.aging_per_year = 30;
    let mut game = game_with(aging, 7);
    let born = game.character().born();
    let mut states = vec![game.view().character.health];
    let mut changed_on = Vec::new();
    while game.summary().is_none() && game.today() < date(1990, 1, 1) {
        assert!(game.advance(DAY_MILLISECONDS), "время идёт, пока жив");
        let state = game.view().character.health;
        if states.last() != Some(&state) {
            states.push(state);
            changed_on.push(game.today());
        }
    }
    assert_eq!(
        states,
        ["здоров", "пошаливает сердце", "плох", "при смерти"],
        "90, 60, 30, 0"
    );
    assert!(
        changed_on
            .iter()
            .all(|day| game.character().has_birthday(*day)),
        "здоровье уходит в дни рождения: {changed_on:?}"
    );
    let ended = game.today();
    assert_eq!(ended.year(), 1986, "третий день рождения — последний");
    assert!(!game.advance(DAY_MILLISECONDS), "мир остановился");
    let view = game.view();
    assert_eq!(view.screen, Screen::Summary);
    assert!(view.time.paused);
    let summary = view.summary.expect("итоги");
    let name = game.character().name().to_uppercase();
    assert_eq!(summary.headline, format!("{name}  {}–1986", born.year()));
    assert!(
        summary
            .source
            .starts_with(&format!("«ВЕЧЕРНЯЯ МОСКВА» · {} · с. ", ended.long())),
        "{}",
        summary.source
    );
    let age = game.character().age_on(ended);
    assert_eq!(
        summary.text,
        format!(
            "Начинал фарцовщиком в общежитии. Скончался {} на {age}-м году жизни.",
            ended.long()
        )
    );
    assert_eq!(
        game.apply(Command::Buy {
            good: "jeans".to_owned(),
            quantity: 1
        }),
        Err(CommandError::LifeOver)
    );
    play(
        &mut game,
        Command::OpenScreen {
            screen: Screen::Market,
        },
    );
}

#[test]
fn an_event_can_end_a_life_and_its_outcome_is_read_first() {
    let mut game = game_with(load(), 7);
    game.start_event(&EventId::new("binge")).expect("запой");
    play(&mut game, Command::Choose { choice: 1 });
    let outcome = game
        .view()
        .pending_event
        .and_then(|event| event.outcome)
        .expect("исход");
    assert_eq!(outcome.effects, "Здоровье: плох.", "90 − 50 — шкала скрыта");
    play(&mut game, Command::Continue);
    assert!(game.summary().is_none(), "ещё жив");

    game.start_event(&EventId::new("shootout"))
        .expect("стрелка");
    play(&mut game, Command::Choose { choice: 1 });
    assert!(game.summary().is_some(), "итоги сняты в день финала");
    assert_eq!(
        game.view().screen,
        Screen::Market,
        "сначала исход события, потом итоги"
    );
    play(&mut game, Command::Continue);
    let view = game.view();
    assert_eq!(view.screen, Screen::Summary);
    assert_eq!(
        view.summary.expect("итоги").text,
        "Начинал фарцовщиком в общежитии. Погиб 1 января 1984, не поладив с бандитами."
    );

    let mut game = game_with(load(), 7);
    live_through(&mut game, "binge");
    game.start_event(&EventId::new("binge"))
        .expect_err("запой отдыхает");
    let mut drinker = load();
    drinker.event_rules.cooldown_days = 1;
    let mut game = game_with(drinker, 7);
    live_through(&mut game, "binge");
    live_until(&mut game, date(1984, 1, 2));
    game.start_event(&EventId::new("binge"))
        .expect("снова запой");
    play(&mut game, Command::Choose { choice: 1 });
    assert_eq!(
        game.summary().map(|summary| summary.ending.as_str()),
        Some("death_health"),
        "здоровье кончилось последствием — финал здоровья"
    );
}

#[test]
fn a_newborn_is_worth_his_cash_and_his_home() {
    let game = game_with(load(), 7);
    assert_eq!(
        game.capital().kopecks(),
        500_000,
        "5 000 р. наличными, а койка в общежитии ничего не стоит"
    );
}

#[test]
fn the_summary_counts_capital_deals_years_and_businesses() {
    let mut game = game_with(load(), 7);
    live_through(&mut game, "windfall");
    for command in [
        Command::Sell {
            good: "chewing_gum".to_owned(),
            quantity: 2,
        },
        Command::Buy {
            good: "jeans".to_owned(),
            quantity: 1,
        },
        Command::Sell {
            good: "jeans".to_owned(),
            quantity: 1,
        },
        Command::OpenBusiness {
            kind: "stall".to_owned(),
        },
    ] {
        play(&mut game, command);
    }
    live_until(&mut game, date(1985, 3, 10));
    let capital_today = game.capital();
    play(
        &mut game,
        Command::Retire {
            ending: "rest".to_owned(),
        },
    );
    let summary = game.summary().expect("итоги").clone();
    assert_eq!(
        summary.capital, capital_today,
        "итоги записывают капитал дня финала"
    );
    let cash = game.player().money().cash().minor_units();
    assert_eq!(
        summary.capital.kopecks(),
        cash + 100_000,
        "наличные и точка палатки по цене открытия"
    );
    let view = game.view();
    assert_eq!(
        summary_lines(&view),
        [
            (
                "Капитал на покое",
                format!("{} р.", grouped((summary.capital.kopecks() + 50) / 100))
            ),
            ("Лучшая сделка", "Жвачка, 1984".to_owned()),
            ("Худшая сделка", "Джинсы, 1984".to_owned()),
            ("Прожито", "1 год 2 месяца".to_owned()),
            ("Дела", "Палатка у метро, Москва".to_owned()),
        ]
        .map(|(label, value)| (label.to_owned(), value))
    );
    let summary_view = view.summary.expect("итоги");
    assert_eq!(
        summary_view.text,
        "Начинал фарцовщиком в общежитии. В 1984 году завёл своё дело — «Палатка у метро». \
         10 марта 1985 отошёл от дел."
    );
    assert_eq!(
        summary_view.headline,
        format!("НА ПОКОЕ: {}", game.character().name().to_uppercase())
    );
}

/// «12 345» — как деньги пишет игра.
fn grouped(rubles: i64) -> String {
    let digits = rubles.to_string();
    let mut groups = Vec::new();
    let mut end = digits.len();
    while end > 3 {
        groups.push(&digits[end - 3..end]);
        end -= 3;
    }
    groups.push(&digits[..end]);
    groups.reverse();
    groups.join(" ")
}

#[test]
fn housing_is_bought_with_the_old_one_in_part_exchange() {
    let mut game = game_with(load(), 7);
    let buy = |housing: &str| Command::BuyHousing {
        housing: housing.to_owned(),
    };
    assert_eq!(
        game.apply(buy("apartment")),
        Err(CommandError::HousingNotOnSale {
            housing: "apartment".to_owned(),
            title: "Квартира в центре".to_owned()
        }),
        "квартиру купят только с приватизацией"
    );
    assert_eq!(
        game.apply(buy("dorm_bed")),
        Err(CommandError::AlreadyLivingThere {
            title: "Койка в общежитии".to_owned()
        })
    );
    assert_eq!(
        game.apply(buy("palace")),
        Err(CommandError::UnknownHousing {
            housing: "palace".to_owned()
        })
    );
    let homes: Vec<(String, Option<String>, bool, bool)> = game
        .view()
        .homes
        .into_iter()
        .map(|home| (home.housing, home.price, home.current, home.available))
        .collect();
    assert_eq!(
        homes,
        [
            ("dorm_bed".to_owned(), None, true, false),
            (
                "communal_room".to_owned(),
                Some("2 000 р.".to_owned()),
                false,
                true
            ),
            (
                "coop_flat".to_owned(),
                Some("9 000 р.".to_owned()),
                false,
                false
            ),
        ],
        "на кооперативную квартиру пока не хватает"
    );

    play(&mut game, buy("communal_room"));
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Переехал: Комната в коммуналке, доплатил 2 000 р.")
    );
    assert_eq!(game.character().housing(), &HousingId::new("communal_room"));
    assert_eq!(
        game.apply(buy("coop_flat")),
        Err(CommandError::NotEnoughCash {
            needed: "7 400 р.".to_owned(),
            cash: "3 000 р.".to_owned()
        }),
        "9 000 р. минус 80% комнаты"
    );

    let mut rich = load();
    for biography in rich.biographies.values_mut() {
        biography.start_cash = speculant_core::economy::Real::from_kopecks(5_000_000);
    }
    let mut game = game_with(rich, 7);
    play(&mut game, buy("coop_flat"));
    play(&mut game, buy("communal_room"));
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Переехал: Комната в коммуналке, выручил 5 200 р."),
        "80% квартиры минус цена комнаты"
    );
    assert_eq!(
        game.player().money().cash().minor_units(),
        4_620_000,
        "50 000 − 9 000 + 5 200"
    );
    live_until(&mut game, date(1992, 7, 1));
    play(&mut game, buy("apartment"));
    assert_eq!(game.view().character.housing, "Квартира в центре");
}

#[test]
fn a_loss_under_the_mattress_pages_urgently() {
    let mut leaky = load();
    leaky
        .housing
        .get_mut(&HousingId::new("dorm_bed"))
        .expect("койка в фикстуре")
        .stash_risk = 1.0;
    leaky.life.pager.devices = vec![PagerDevice {
        good: GoodId::new("chewing_gum"),
        title: "ПЕЙДЖЕР · абонент {number}".to_owned(),
    }];
    let mut game = game_with(leaky, 7);
    play(
        &mut game,
        Command::Buy {
            good: "chewing_gum".to_owned(),
            quantity: 1,
        },
    );
    play(
        &mut game,
        Command::Deposit {
            place: "mattress".to_owned(),
            amount: 1000,
        },
    );
    live_until(&mut game, date(1984, 2, 1));
    let pager = game.view().pager.expect("пейджер в кармане");
    let loss = pager.messages.last().expect("потеря на пейджере");
    assert_eq!(
        (loss.text.as_str(), loss.urgent),
        ("КОЙКА В ОБЩЕЖИТИИ: ИЗ-ПОД МАТРАСА ПРОПАЛО 1 000 Р.", true)
    );
}

#[test]
fn a_poor_home_loses_the_money_under_the_mattress() {
    let mut leaky = load();
    leaky
        .housing
        .get_mut(&HousingId::new("dorm_bed"))
        .expect("койка в фикстуре")
        .stash_risk = 1.0;
    let mut game = game_with(leaky, 7);
    play(
        &mut game,
        Command::Deposit {
            place: "mattress".to_owned(),
            amount: 1000,
        },
    );
    live_until(&mut game, date(1984, 2, 1));
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Койка в общежитии: из-под матраса пропало 1 000 р.")
    );
    assert!(
        game.player()
            .money()
            .balance(&speculant_core::content::MoneyPlaceId::new("mattress"))
            .is_zero()
    );
    assert_eq!(game.player().money().cash().minor_units(), 400_000);
}

#[test]
fn voluntary_departures_open_with_their_time_and_speak_the_language_of_their_epoch() {
    let mut game = game_with(load(), 7);
    let retire = |ending: &str| Command::Retire {
        ending: ending.to_owned(),
    };
    let exits: Vec<(String, bool)> = game
        .view()
        .exits
        .into_iter()
        .map(|exit| (exit.ending, exit.available))
        .collect();
    assert_eq!(
        exits,
        [("emigration".to_owned(), false), ("rest".to_owned(), true)]
    );
    assert_eq!(
        game.apply(retire("emigration")),
        Err(CommandError::EndingUnavailable {
            ending: "emigration".to_owned(),
            title: "Эмигрировать".to_owned()
        }),
        "из СССР 1984 года не уехать"
    );
    assert_eq!(
        game.apply(retire("prison")),
        Err(CommandError::EndingUnavailable {
            ending: "prison".to_owned(),
            title: "Срок".to_owned()
        }),
        "в зону по своей воле не уходят"
    );
    assert_eq!(
        game.apply(retire("heaven")),
        Err(CommandError::UnknownEnding {
            ending: "heaven".to_owned()
        })
    );
    play(
        &mut game,
        Command::Travel {
            destination: "leningrad".to_owned(),
        },
    );
    assert_eq!(game.apply(retire("rest")), Err(CommandError::Traveling));

    live_until(&mut game, date(1992, 3, 1));
    play(&mut game, retire("emigration"));
    let summary = game.view().summary.expect("итоги");
    assert!(
        summary
            .source
            .starts_with("«МОСКОВСКИЙ КОМСОМОЛЕЦ» · 1 марта 1992 · с. "),
        "в дикий рынок пишет другая газета: {}",
        summary.source
    );
    assert_eq!(summary.lines[0].label, "Вывезено за границу");
    assert!(
        summary.lines[0].value.starts_with("$ "),
        "с 1992 года капитал считают в долларах: {}",
        summary.lines[0].value
    );
    assert_eq!(summary.lines[3].value, "8 лет 2 месяца");

    play(&mut game, Command::NewLife(settings("student", 9)));
    let view = game.view();
    assert_eq!(game.today(), GameDate::START);
    assert_eq!(view.summary, None);
    assert_eq!(view.screen, Screen::Market);
}
