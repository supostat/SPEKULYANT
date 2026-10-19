//! Своё дело (§7): точки в форме собственности своего дня, управляющие из шаблонов людей, шесть
//! политик и прогноз макета «Управляющий», расчёт месяца, происшествия, которые политика закрывает
//! сама, и те, что приходят хозяину событием, и дела соперников.

use std::path::Path;
use std::sync::Arc;

use speculant_core::business::BusinessId;
use speculant_core::content::{BusinessKindId, CityId, Content, GoodId, HabitId};
use speculant_core::life::PagerDevice;
use speculant_core::npc::Strategy;
use speculant_core::time::GameDate;
use speculant_core::view::{BusinessView, LevelView, PagerMessageView, StateView};
use speculant_core::{Command, CommandError, Game, LifeSettings};

const BUSINESS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/business");
const DAY_MILLISECONDS: u32 = 1800;
const STALL: &str = "stall@moscow";
const KIOSKS: &str = "kiosks@moscow";
const POLICIES: [&str; 6] = [
    "markup",
    "roof",
    "purchase",
    "night_trade",
    "inspection",
    "wage",
];
const RISK_LEVELS: [&str; 5] = ["низкий", "умеренный", "средний", "высокий", "очень высокий"];

fn load() -> Content {
    Content::load(Path::new(BUSINESS)).expect("фикстура дел валидна")
}

fn game_with(content: Content, seed: u64) -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, Arc::new(content)).expect("жизнь начинается")
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn play(game: &mut Game, command: Command) {
    let shown = format!("{command:?}");
    game.apply(command)
        .unwrap_or_else(|error| panic!("{shown}: {error}"));
}

fn set_policy(game: &mut Game, business: &str, policy: &str, option: usize) {
    play(
        game,
        Command::SetPolicy {
            business: business.to_owned(),
            policy: policy.to_owned(),
            option,
        },
    );
}

fn open(game: &mut Game, kind: &str) -> Result<(), CommandError> {
    game.apply(Command::OpenBusiness {
        kind: kind.to_owned(),
    })
}

fn hire(game: &mut Game, business: &str, candidate: &str) -> Result<(), CommandError> {
    game.apply(Command::Hire {
        business: business.to_owned(),
        candidate: candidate.to_owned(),
    })
}

/// Живёт по дню до даты; в тихой фикстуре событий нет.
fn live_until(game: &mut Game, until: GameDate) {
    while game.today() < until {
        assert!(game.advance(DAY_MILLISECONDS), "время идёт");
        assert!(
            game.events().pending().is_none(),
            "в тихой фикстуре событий нет"
        );
    }
}

fn day(game: &mut Game) {
    assert!(game.advance(DAY_MILLISECONDS), "время идёт");
}

fn business<'a>(view: &'a StateView, id: &str) -> &'a BusinessView {
    view.businesses
        .iter()
        .find(|business| business.business == id)
        .unwrap_or_else(|| panic!("дела {id} нет"))
}

fn report_lines(view: &StateView, id: &str) -> Vec<(String, String)> {
    business(view, id)
        .report
        .as_ref()
        .expect("отчёт за прошлый месяц")
        .lines
        .iter()
        .map(|line| (line.label.clone(), line.value.clone()))
        .collect()
}

fn lines(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(label, value)| ((*label).to_owned(), (*value).to_owned()))
        .collect()
}

fn cash_kopecks(game: &Game) -> i64 {
    game.player().money().cash().minor_units()
}

/// Палатка у метро с Геной Кабановым с первого дня 1984 года.
fn stall_with_gena(content: Content) -> Game {
    let mut game = game_with(content, 7);
    open(&mut game, "stall").expect("палатку можно открыть с 1984 года");
    hire(&mut game, STALL, "gena_kabanov").expect("Гена свободен");
    game
}

/// Ларьки у «Тушинской» в три точки с Геной Кабановым, нанятым в 1994 году, — дело макета
/// «Управляющий».
fn mockup_kiosks() -> Game {
    let mut game = game_with(load(), 7);
    live_until(&mut game, date(1994, 9, 9));
    for _ in 0..3 {
        open(&mut game, "kiosks").expect("в 1994 году ларьки открывают как ТОО");
    }
    hire(&mut game, KIOSKS, "gena_kabanov").expect("Гена свободен");
    game
}

/// Положения стрелок макета: наценка 30%, крыша 15%, закупка сам, ночью «Рояль», звонить мне,
/// зарплата +10%.
const MOCKUP_POSITIONS: [usize; 6] = [1, 2, 0, 2, 1, 2];

/// Формула прогноза из макета дизайн-книги (§VII, `drawManager`) — эталон для ядра: сдвиг
/// выручки в процентах и уровни наезда и воровства от 0 до 4.
fn mockup_forecast(positions: [usize; 6]) -> (i32, usize, usize) {
    let [markup, roof, purchase, night, inspection, wage] = positions;
    let step = |position: usize| i32::try_from(position).expect("положение стрелки — 0, 1 или 2");
    let markup_factor = [0.8, 1.0, 1.25][markup];
    let through_gena = purchase == 2;
    let revenue = ((markup_factor - 1.0) * 40.0 + f64::from(step(night)) * 9.0
        - if through_gena { 6.0 } else { 0.0 })
    .round() as i32;
    let raid = 2 - step(roof) + i32::from(night == 2) + i32::from(inspection == 2)
        - i32::from(inspection == 0);
    let theft = 2 - step(wage) + i32::from(through_gena);
    let level = |steps: i32| usize::try_from(steps.clamp(0, 4)).expect("уровень от нуля");
    (revenue, level(raid), level(theft))
}

#[test]
fn a_business_opens_in_the_form_of_its_day_and_grows_by_points() {
    let mut game = game_with(load(), 7);
    let kinds: Vec<(String, String, String, bool)> = game
        .view()
        .business_kinds
        .into_iter()
        .map(|kind| (kind.kind, kind.form, kind.cost, kind.available))
        .collect();
    assert_eq!(
        kinds,
        [
            (
                "sewing_workshop".into(),
                "Подпольный цех".into(),
                "1 500 р.".into(),
                true
            ),
            ("stall".into(), "Артель".into(), "1 000 р.".into(), true),
        ],
        "ларьков в 1984 году ещё нет: кооперация — с 1988-го"
    );
    assert_eq!(
        open(&mut game, "kiosks"),
        Err(CommandError::BusinessNotYet {
            kind: "kiosks".to_owned(),
            title: "Ларьки у «Тушинской»".to_owned()
        })
    );
    assert_eq!(
        open(&mut game, "casino"),
        Err(CommandError::UnknownBusinessKind {
            kind: "casino".to_owned()
        })
    );

    open(&mut game, "stall").expect("палатка открывается");
    let view = game.view();
    assert_eq!(view.header.money, ["Наличные: 99 000 р."]);
    assert_eq!(
        view.messages.last().map(String::as_str),
        Some("Открыл: Палатка у метро, Москва за 1 000 р.")
    );
    let stall = business(&view, STALL);
    assert_eq!(stall.title, "Палатка у метро · 1 точка");
    assert_eq!(stall.city, "Москва");
    assert_eq!(stall.form, "Артель");
    assert_eq!(stall.manager, None, "дело стоит без управляющего");
    assert_eq!(stall.forecast, None);
    assert_eq!(stall.report, None);
    let defaults: Vec<(&str, usize)> = stall
        .policies
        .iter()
        .map(|policy| (policy.policy.as_str(), policy.selected))
        .collect();
    assert_eq!(
        defaults,
        [
            ("markup", 1),
            ("roof", 0),
            ("purchase", 0),
            ("night_trade", 0),
            ("inspection", 1),
            ("wage", 1),
        ]
    );

    open(&mut game, "stall").expect("вторая точка расширяет дело");
    assert_eq!(
        business(&game.view(), STALL).title,
        "Палатка у метро · 2 точки"
    );
    assert_eq!(game.businesses().len(), 1);
    assert_eq!(
        open(&mut game, "stall"),
        Err(CommandError::MaxPoints {
            kind: "stall".to_owned(),
            title: "Палатка у метро".to_owned(),
            max: 2
        })
    );
    assert!(
        !game
            .view()
            .business_kinds
            .iter()
            .any(|kind| kind.kind == "stall" && kind.available),
        "третьей точки не будет"
    );

    let mut poor = load();
    for biography in poor.biographies.values_mut() {
        biography.start_cash = speculant_core::economy::Real::from_kopecks(50_000);
    }
    let mut poor = game_with(poor, 7);
    assert_eq!(
        open(&mut poor, "stall"),
        Err(CommandError::NotEnoughCash {
            needed: "1 000 р.".to_owned(),
            cash: "500 р.".to_owned()
        })
    );

    play(
        &mut game,
        Command::Travel {
            destination: "leningrad".to_owned(),
        },
    );
    assert_eq!(
        open(&mut game, "sewing_workshop"),
        Err(CommandError::Traveling)
    );
    assert!(
        game.view().business_kinds.is_empty(),
        "в дороге ничего не открыть"
    );
}

#[test]
fn the_manager_screen_shows_the_mockup() {
    let mut game = mockup_kiosks();
    for (policy, option) in POLICIES.into_iter().zip(MOCKUP_POSITIONS) {
        set_policy(&mut game, KIOSKS, policy, option);
    }
    let view = game.view();
    let kiosks = business(&view, KIOSKS);
    assert_eq!(kiosks.title, "Ларьки у «Тушинской» · 3 точки");
    assert_eq!(kiosks.form, "ТОО");
    let manager = kiosks.manager.as_ref().expect("Гена управляет");
    assert_eq!(manager.name, "Гена Кабанов");
    assert_eq!(manager.since, "с 1994 года");
    let level = |label: &str, level| LevelView {
        label: label.to_owned(),
        level,
        scale: 5,
    };
    assert_eq!(
        manager.traits,
        [
            level("Честность", 3),
            level("Хватка", 4),
            level("Лояльность", 2)
        ]
    );
    assert_eq!(manager.habits, ["Пьёт по пятницам.", "Брат в милиции."]);
    let rows: Vec<(&str, &str)> = kiosks
        .policies
        .iter()
        .map(|policy| {
            (
                policy.label.as_str(),
                policy.options[policy.selected].as_str(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        [
            ("Наценка", "30%"),
            ("Платить крыше", "да, 15%"),
            ("Закупка", "сам, Черкизон"),
            ("Ночная торговля", "водка «Рояль»"),
            ("При проверке", "звонить мне"),
            ("Зарплата", "+10% к рынку"),
        ]
    );
    let forecast = kiosks.forecast.as_ref().expect("прогноз с управляющим");
    assert_eq!(forecast.revenue, "+18%");
    assert!(!forecast.revenue_falls);
    assert_eq!(forecast.raid_risk, "умеренный");
    assert!(!forecast.raid_risk_high);
    assert_eq!(forecast.theft_risk, "низкий");
    assert!(!forecast.theft_risk_high);
}

#[test]
fn the_forecast_follows_the_mockup_formula_for_every_policy_position() {
    let mut game = mockup_kiosks();
    for combination in 0..3_usize.pow(6) {
        let mut positions = [0; 6];
        let mut rest = combination;
        for position in &mut positions {
            *position = rest % 3;
            rest /= 3;
        }
        for (policy, option) in POLICIES.into_iter().zip(positions) {
            set_policy(&mut game, KIOSKS, policy, option);
        }
        let (revenue, raid, theft) = mockup_forecast(positions);
        let view = game.view();
        let forecast = business(&view, KIOSKS)
            .forecast
            .clone()
            .expect("прогноз с управляющим");
        let sign = if revenue < 0 { '−' } else { '+' };
        assert_eq!(
            forecast.revenue,
            format!("{sign}{}%", revenue.unsigned_abs()),
            "выручка при {positions:?}"
        );
        assert_eq!(forecast.revenue_falls, revenue < 0, "{positions:?}");
        assert_eq!(
            forecast.raid_risk, RISK_LEVELS[raid],
            "наезд при {positions:?}"
        );
        assert_eq!(forecast.raid_risk_high, raid >= 3, "{positions:?}");
        assert_eq!(
            forecast.theft_risk, RISK_LEVELS[theft],
            "воровство при {positions:?}"
        );
        assert_eq!(forecast.theft_risk_high, theft >= 3, "{positions:?}");
    }
}

#[test]
fn hiring_and_firing_follow_the_people() {
    let mut game = game_with(load(), 7);
    open(&mut game, "stall").expect("палатка открывается");
    let candidates: Vec<String> = game
        .view()
        .candidates
        .into_iter()
        .map(|candidate| candidate.candidate)
        .collect();
    assert_eq!(
        candidates,
        ["gena_kabanov", "tamara_ivanovna", "vitya_kosoy"],
        "соперники в управляющие не идут"
    );
    assert_eq!(
        hire(&mut game, "kiosks@moscow", "gena_kabanov"),
        Err(CommandError::UnknownBusiness {
            business: "kiosks@moscow".to_owned()
        })
    );
    assert_eq!(
        hire(&mut game, STALL, "misha_trader"),
        Err(CommandError::UnknownCandidate {
            candidate: "misha_trader".to_owned()
        })
    );

    hire(&mut game, STALL, "gena_kabanov").expect("Гена свободен");
    let view = game.view();
    assert_eq!(
        view.messages.last().map(String::as_str),
        Some("Палатка у метро, Москва: управляющий — Гена Кабанов")
    );
    assert!(
        !view
            .candidates
            .iter()
            .any(|candidate| candidate.candidate == "gena_kabanov")
    );
    let gena = view
        .contacts
        .iter()
        .find(|contact| contact.contact == "gena_kabanov")
        .expect("нанятый — в связях");
    assert_eq!(gena.history, ["01.01.1984: Нанят: Палатка у метро, Москва"]);

    open(&mut game, "sewing_workshop").expect("цех открывается");
    assert_eq!(
        hire(&mut game, "sewing_workshop@moscow", "gena_kabanov"),
        Err(CommandError::CandidateBusy {
            candidate: "gena_kabanov".to_owned(),
            name: "Гена Кабанов".to_owned(),
            business: "Палатка у метро, Москва".to_owned()
        })
    );
    assert_eq!(
        hire(&mut game, STALL, "tamara_ivanovna"),
        Err(CommandError::ManagerInPlace {
            business: "Палатка у метро, Москва".to_owned(),
            name: "Гена Кабанов".to_owned()
        })
    );
    let fire = |employee: &str| Command::Fire {
        business: STALL.to_owned(),
        employee: employee.to_owned(),
    };
    assert_eq!(
        game.apply(fire("tamara_ivanovna")),
        Err(CommandError::NotManager {
            business: STALL.to_owned(),
            employee: "tamara_ivanovna".to_owned()
        })
    );

    play(&mut game, fire("gena_kabanov"));
    let view = game.view();
    assert_eq!(business(&view, STALL).manager, None);
    assert_eq!(
        view.messages.last().map(String::as_str),
        Some("Палатка у метро, Москва: Гена Кабанов уволен")
    );
    let gena = view
        .contacts
        .iter()
        .find(|contact| contact.contact == "gena_kabanov")
        .expect("уволенный помнит");
    assert_eq!(gena.relation, "обижен");
    assert_eq!(
        gena.history.last().map(String::as_str),
        Some("01.01.1984: Уволен")
    );
    assert!(
        !view
            .candidates
            .iter()
            .any(|candidate| candidate.candidate == "gena_kabanov")
    );
    assert_eq!(
        hire(&mut game, STALL, "gena_kabanov"),
        Err(CommandError::CandidateRefuses {
            candidate: "gena_kabanov".to_owned(),
            name: "Гена Кабанов".to_owned(),
            reason: "обижен и помогать не станет".to_owned()
        })
    );
    hire(&mut game, STALL, "tamara_ivanovna").expect("место свободно");
}

#[test]
fn policies_are_chosen_by_number_among_the_policies_of_the_kind() {
    let mut game = game_with(load(), 7);
    open(&mut game, "sewing_workshop").expect("цех открывается");
    let workshop = "sewing_workshop@moscow";
    let set = |policy: &str, option| Command::SetPolicy {
        business: workshop.to_owned(),
        policy: policy.to_owned(),
        option,
    };
    assert_eq!(
        game.apply(set("night_trade", 1)),
        Err(CommandError::UnknownPolicy {
            business: workshop.to_owned(),
            policy: "night_trade".to_owned()
        }),
        "в цеху ночной торговли нет"
    );
    assert_eq!(
        game.apply(set("discount", 0)),
        Err(CommandError::UnknownPolicy {
            business: workshop.to_owned(),
            policy: "discount".to_owned()
        })
    );
    assert_eq!(
        game.apply(set("markup", 3)),
        Err(CommandError::NoSuchPolicyOption {
            policy: "markup".to_owned(),
            option: 3,
            count: 3
        })
    );
    play(&mut game, set("markup", 2));
    let view = game.view();
    let markup = business(&view, workshop)
        .policies
        .iter()
        .find(|policy| policy.policy == "markup")
        .expect("наценка у цеха есть");
    assert_eq!(markup.options[markup.selected], "45%");
    assert_eq!(
        business(&view, workshop).policies.len(),
        5,
        "у цеха пять политик из шести"
    );
}

#[test]
fn a_month_pays_the_owner_income_minus_costs() {
    let mut game = stall_with_gena(load());
    live_until(&mut game, date(1984, 2, 29));
    let before = cash_kopecks(&game);
    day(&mut game);
    assert_eq!(
        cash_kopecks(&game) - before,
        65_100,
        "900 р. × хватка 1,1 = 990 р. минус налог артели 10%, аренда и зарплата"
    );
    let view = game.view();
    let report = business(&view, STALL).report.as_ref().expect("отчёт");
    assert_eq!(report.month, "февраль 1984");
    assert_eq!(
        report_lines(&view, STALL),
        lines(&[
            ("Выручка", "990 р."),
            ("Налог", "99 р."),
            ("Аренда", "40 р."),
            ("Зарплата", "200 р."),
            ("Итог", "+651 р."),
        ])
    );
    assert!(
        view.messages
            .contains(&"Палатка у метро, Москва: февраль 1984 — прибыль 651 р.".to_owned())
    );

    set_policy(&mut game, STALL, "markup", 2);
    set_policy(&mut game, STALL, "wage", 2);
    live_until(&mut game, date(1984, 4, 1));
    assert_eq!(
        report_lines(&game.view(), STALL),
        lines(&[
            ("Выручка", "1 089 р."),
            ("Налог", "109 р."),
            ("Аренда", "40 р."),
            ("Зарплата", "220 р."),
            ("Итог", "+720 р."),
        ]),
        "наценка 45% даёт +10% выручки, зарплата +10% к рынку стоит 20 р.; от 10 р. — без копеек"
    );
}

#[test]
fn an_idle_business_pays_rent_and_a_loss_it_cannot_cover_closes_it() {
    let mut game = game_with(load(), 7);
    open(&mut game, "stall").expect("палатка открывается");
    live_until(&mut game, date(1984, 2, 29));
    let before = cash_kopecks(&game);
    day(&mut game);
    assert_eq!(cash_kopecks(&game) - before, -4_000);
    assert_eq!(
        report_lines(&game.view(), STALL),
        lines(&[
            ("Выручка", "0 р."),
            ("Аренда", "40 р."),
            ("Зарплата", "0 р."),
            ("Итог", "−40 р."),
        ])
    );

    let mut broke = load();
    for biography in broke.biographies.values_mut() {
        biography.start_cash = speculant_core::economy::Real::from_kopecks(100_000);
    }
    let mut game = game_with(broke, 7);
    open(&mut game, "stall").expect("на точку хватает впритык");
    assert_eq!(cash_kopecks(&game), 0);
    live_until(&mut game, date(1984, 2, 1));
    assert!(game.businesses().is_empty(), "долг закрыл дело");
    assert_eq!(cash_kopecks(&game), 0);
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Палатка у метро, Москва: январь 1984 — убыток 39 р., покрыть нечем — дело закрыто"),
        "аренда 30 дней из 31 — 38,71 р."
    );
}

/// Наезд каждый день: шанс беды на любом уровне риска — единица; `racket_share` артели делит
/// его между рэкетом и проверкой.
fn raids_every_day(racket_share: f64) -> Content {
    let mut content = load();
    for level in &mut content.business.risk_levels {
        level.monthly_chance = 1.0;
    }
    content
        .business_forms
        .get_mut(&speculant_core::content::BusinessFormId::new("artel"))
        .expect("артель в фикстуре")
        .racket_share = racket_share;
    content
}

#[test]
fn incidents_inside_the_policies_resolve_without_the_owner() {
    let mut game = stall_with_gena(raids_every_day(1.0));
    set_policy(&mut game, STALL, "roof", 1);
    day(&mut game);
    assert!(game.events().pending().is_none(), "крыша закрыла рэкет");
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Палатка у метро, Москва: наезд — крыша разобралась")
    );

    let mut game = stall_with_gena(raids_every_day(0.0));
    set_policy(&mut game, STALL, "inspection", 0);
    day(&mut game);
    assert!(
        game.events().pending().is_none(),
        "управляющий откупился сам"
    );
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Палатка у метро, Москва: проверка — откупились за 99 р."),
        "откуп — десятая часть месячной выручки"
    );

    set_policy(&mut game, STALL, "inspection", 2);
    day(&mut game);
    assert_eq!(game.today(), date(1984, 1, 3));
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Палатка у метро, Москва: проверка — закрылись до 07.01.1984")
    );
    let quiet = game.view().messages.len();
    live_until(&mut game, date(1984, 1, 6));
    let view = game.view();
    assert_eq!(view.messages.len(), quiet, "закрытое дело не трогают");
    assert_eq!(
        business(&view, STALL).closed_until.as_deref(),
        Some("закрыто до 07.01.1984")
    );
    day(&mut game);
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Палатка у метро, Москва: проверка — закрылись до 11.01.1984"),
        "открылись — и проверка снова тут"
    );
    live_until(&mut game, date(1984, 2, 1));
    let report = report_lines(&game.view(), STALL);
    assert!(
        report.contains(&("Откупы".to_owned(), "99 р.".to_owned())),
        "откуп первого дня ушёл из выручки января: {report:?}"
    );
}

#[test]
fn incidents_outside_the_policies_come_to_the_owner_as_events() {
    let mut restless = raids_every_day(1.0);
    restless.business.departure_chances = vec![1.0; 5];
    let mut game = stall_with_gena(restless);
    day(&mut game);
    let pending = game
        .events()
        .pending()
        .expect("рэкет без крыши — вопрос к хозяину");
    assert_eq!(pending.event.as_str(), "racket_demand");
    assert_eq!(
        pending.business,
        BusinessId::parse(STALL),
        "событие про это дело"
    );
    let shown = game.view().pending_event.expect("событие на экране");
    assert_eq!(
        shown.text,
        "Гена Кабанов звонит: «Хозяин, пришли крепкие ребята, хотят долю». Дело: Палатка у метро, Москва."
    );
    play(&mut game, Command::Choose { choice: 1 });
    let outcome = game
        .view()
        .pending_event
        .and_then(|event| event.outcome)
        .expect("исход");
    assert_eq!(
        outcome.text,
        "У дела «Палатка у метро, Москва» теперь есть крыша."
    );
    assert_eq!(
        outcome.effects,
        "Бандиты +1. Палатка у метро, Москва: Платить крыше — да, 10%."
    );
    play(&mut game, Command::Continue);
    assert_eq!(
        game.journal().len(),
        1,
        "уход управляющего в тот же день прошёл мимо"
    );

    day(&mut game);
    assert_eq!(
        game.view().messages.last().map(String::as_str),
        Some("Палатка у метро, Москва: наезд — крыша разобралась"),
        "теперь рэкет — забота крыши"
    );
    assert_eq!(
        game.events()
            .pending()
            .map(|pending| pending.event.as_str()),
        Some("manager_departure"),
        "а уход управляющего политиками не закрыть"
    );

    let mut game = stall_with_gena(raids_every_day(0.0));
    day(&mut game);
    let pending = game
        .events()
        .pending()
        .expect("проверка, а политика — звонить мне");
    assert_eq!(pending.event.as_str(), "inspection_call");
    play(&mut game, Command::Choose { choice: 2 });
    let outcome = game
        .view()
        .pending_event
        .and_then(|event| event.outcome)
        .expect("исход");
    assert_eq!(
        outcome.effects,
        "Палатка у метро, Москва: закрыто до 06.01.1984."
    );
    play(&mut game, Command::Continue);
    assert_eq!(
        business(&game.view(), STALL).closed_until.as_deref(),
        Some("закрыто до 06.01.1984")
    );
}

#[test]
fn an_incident_that_needs_the_owner_pages_him_urgently() {
    let mut paged = raids_every_day(1.0);
    paged.life.pager.devices = vec![PagerDevice {
        good: GoodId::new("chewing_gum"),
        title: "ПЕЙДЖЕР · абонент {number}".to_owned(),
    }];
    let mut game = stall_with_gena(paged);
    play(
        &mut game,
        Command::Buy {
            good: "chewing_gum".to_owned(),
            quantity: 1,
        },
    );
    day(&mut game);
    let pager = game.view().pager.expect("пейджер в кармане");
    assert_eq!(
        pager.messages.last(),
        Some(&PagerMessageView {
            time: "02.01".to_owned(),
            text: "ПАЛАТКА У МЕТРО, МОСКВА: НАЕЗД".to_owned(),
            urgent: true,
        })
    );
}

#[test]
fn the_manager_may_leave_and_a_fire_takes_the_business() {
    let mut leaving = load();
    leaving.business.departure_chances = vec![1.0; 5];
    let mut game = stall_with_gena(leaving.clone());
    day(&mut game);
    assert_eq!(
        game.events()
            .pending()
            .map(|pending| pending.event.as_str()),
        Some("manager_departure")
    );
    play(&mut game, Command::Choose { choice: 1 });
    let outcome = game
        .view()
        .pending_event
        .and_then(|event| event.outcome)
        .expect("исход");
    assert_eq!(
        outcome.text, "Гена Кабанов собирает вещи.",
        "имя снято в начале события"
    );
    assert_eq!(
        outcome.effects,
        "Палатка у метро, Москва: Гена Кабанов уходит."
    );
    play(&mut game, Command::Continue);
    let view = game.view();
    assert_eq!(business(&view, STALL).manager, None);
    assert!(
        view.candidates
            .iter()
            .any(|candidate| candidate.candidate == "gena_kabanov"),
        "ушёл сам — не обижен, можно позвать снова"
    );

    let mut burning = load();
    burning.business.fire_chance = 1.0;
    let mut game = stall_with_gena(burning);
    day(&mut game);
    play(&mut game, Command::Choose { choice: 1 });
    let outcome = game
        .view()
        .pending_event
        .and_then(|event| event.outcome)
        .expect("исход");
    assert_eq!(
        outcome.text,
        "Гена Кабанов разводит руками над тем, что было делом «Палатка у метро, Москва»."
    );
    assert_eq!(outcome.effects, "Палатка у метро, Москва: дело потеряно.");
    play(&mut game, Command::Continue);
    assert!(game.businesses().is_empty());

    let mut silent = leaving;
    silent.events.clear();
    let mut game = stall_with_gena(silent);
    live_until(&mut game, date(1984, 1, 10));
    assert!(
        business(&game.view(), STALL).manager.is_some(),
        "без события про уход происшествие проходит мимо"
    );
}

#[test]
fn theft_and_drink_follow_the_traits_of_the_manager() {
    let mut thieving = load();
    for (level, chance) in thieving
        .business
        .risk_levels
        .iter_mut()
        .zip([0.0, 0.0, 0.0, 1.0, 1.0])
    {
        level.monthly_chance = chance;
    }
    let mut game = game_with(thieving, 7);
    open(&mut game, "stall").expect("палатка открывается");
    hire(&mut game, STALL, "vitya_kosoy").expect("Витя свободен");
    set_policy(&mut game, STALL, "wage", 0);
    let forecast = business(&game.view(), STALL)
        .forecast
        .clone()
        .expect("прогноз");
    assert_eq!(
        forecast.theft_risk, "очень высокий",
        "честность 1, зарплата ниже рынка"
    );
    assert!(forecast.theft_risk_high);
    assert_eq!(forecast.raid_risk, "средний");
    live_until(&mut game, date(1984, 3, 1));
    assert_eq!(
        report_lines(&game.view(), STALL),
        lines(&[
            ("Выручка", "990 р."),
            ("Налог", "99 р."),
            ("Аренда", "40 р."),
            ("Зарплата", "180 р."),
            ("Недостача", "149 р."),
            ("Итог", "+523 р."),
        ])
    );

    let mut drinking = load();
    drinking
        .habits
        .get_mut(&HabitId::new("drinks_on_fridays"))
        .expect("привычка Гены")
        .skill_loss_chance = 1.0;
    let mut game = stall_with_gena(drinking);
    live_until(&mut game, date(1984, 2, 1));
    assert!(
        game.view()
            .messages
            .contains(&"Гена Кабанов пьёт: хватка 3 из 5".to_owned())
    );
    live_until(&mut game, date(1984, 3, 1));
    let view = game.view();
    let manager = business(&view, STALL)
        .manager
        .as_ref()
        .expect("Гена при деле");
    assert_eq!(manager.traits[1].label, "Хватка");
    assert_eq!(manager.traits[1].level, 2, "два месяца — два деления");
}

#[test]
fn rivals_open_businesses_and_share_the_market() {
    let crowded_content = || {
        let mut content = load();
        content.rival_rules.min_rivals = 2;
        content.rival_rules.max_rivals = 2;
        content
            .rival_rules
            .strategies
            .get_mut(&Strategy::Trader)
            .expect("торговцы в правилах")
            .business_chance = 1.0;
        content
    };
    let stall = BusinessKindId::new("stall");
    let moscow = CityId::new("moscow");
    let seed = (1..=200)
        .find(|seed| {
            let mut game = game_with(crowded_content(), *seed);
            live_until(&mut game, date(1984, 3, 1));
            game.rivals().business_points(&stall, &moscow) > 0
        })
        .expect("в одном из миров соперник держит палатки в Москве");

    let mut crowded = game_with(crowded_content(), seed);
    open(&mut crowded, "stall").expect("палатка открывается");
    hire(&mut crowded, STALL, "gena_kabanov").expect("Гена свободен");
    live_until(&mut crowded, date(1984, 4, 1));
    for rival in crowded.rivals().iter() {
        assert!(
            !rival.businesses().is_empty(),
            "{} открыл дело",
            rival.npc()
        );
        assert!(
            rival
                .capital(crowded.content(), crowded.markets(), crowded.today())
                .kopecks()
                > 4_000_000,
            "точки — тоже капитал"
        );
    }
    let competitors = crowded.rivals().business_points(&stall, &moscow);
    assert!(competitors > 0);
    let income = crowded.businesses()[&BusinessId::parse(STALL).expect("идентификатор дела")]
        .last_report()
        .expect("отчёт за март")
        .income
        .minor_units();
    let alone = 99_000.0;
    assert_eq!(
        income,
        (alone / (1.0 + 0.1 * f64::from(competitors))).round() as i64,
        "{competitors} точек соперников делят выручку 990 р."
    );
}
