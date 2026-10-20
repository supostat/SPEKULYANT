//! Экономика (§5, §6) на фикстуре из графика дизайн-книги (§V): кривые цен по эпохам, номинал
//! через индекс и деноминацию, множители городов, запас и спрос толкучки, шоки, валюты, места
//! хранения денег и настоящие команды торговли и вкладов.

use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{CityId, Content, GoodId, MoneyPlaceId};
use speculant_core::economy::{CityMarket, Currency, MarketRules, Markets, Money, Real, Trader};
use speculant_core::rng::WorldRng;
use speculant_core::time::GameDate;
use speculant_core::view::{MoneyPlaceView, PocketItemView};
use speculant_core::{Command, CommandError, Game, LifeSettings};

const ECONOMY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/economy");
/// Дрейф цен в фикстуре выключен: сид дрейфа рынков ни на что не влияет.
const DRIFT_SEED: u64 = 1;

fn content() -> Arc<Content> {
    Arc::new(Content::load(Path::new(ECONOMY)).expect("фикстура экономики валидна"))
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn rubles(rubles: i64) -> Real {
    Real::from_kopecks(rubles * 100)
}

fn good(id: &str) -> GoodId {
    GoodId::new(id)
}

fn city(id: &str) -> CityId {
    CityId::new(id)
}

fn place(id: &str) -> MoneyPlaceId {
    MoneyPlaceId::new(id)
}

fn new_game() -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed: 7,
    };
    Game::new_life(&settings, content()).expect("жизнь начинается")
}

fn buy(good: &str, quantity: u32) -> Command {
    Command::Buy {
        good: good.to_owned(),
        quantity,
    }
}

fn sell(good: &str, quantity: u32) -> Command {
    Command::Sell {
        good: good.to_owned(),
        quantity,
    }
}

fn deposit(place: &str, amount: u64) -> Command {
    Command::Deposit {
        place: place.to_owned(),
        amount,
    }
}

fn withdraw(place: &str, amount: u64) -> Command {
    Command::Withdraw {
        place: place.to_owned(),
        amount,
    }
}

fn total(tags: &[Money]) -> Money {
    tags.iter()
        .fold(Money::zero(Currency::Ruble), |sum, tag| sum.plus(*tag))
}

#[test]
fn a_video_recorder_costs_half_a_car_in_1986_and_kopecks_by_2000() {
    let content = content();
    let curve = &content.goods[&good("video_recorder")].curve;
    let in_1986 = curve
        .kopecks_on(date(1986, 1, 1))
        .expect("видак уже продают");
    let in_2000 = curve
        .kopecks_on(date(2000, 1, 1))
        .expect("видак ещё продают");
    assert_eq!(in_1986, 120_000.0, "1 200 рублей 1984 года");
    assert_eq!(in_2000, 12_000.0, "в десять раз дешевле");
    let rules = &content.money;
    let tag_1986 = rules.price_tag(in_1986, date(1986, 1, 1));
    let tag_2000 = rules.price_tag(in_2000, date(2000, 1, 1));
    assert_eq!(rules.format_money(tag_1986, date(1986, 1, 1)), "1 224 р.");
    assert_eq!(rules.format_money(tag_2000, date(2000, 1, 1)), "5 400 р.");
}

#[test]
fn between_points_a_price_changes_by_the_same_factor_every_day() {
    let content = content();
    let curve = &content.goods[&good("video_recorder")].curve;
    let price = |day: u32| {
        curve
            .kopecks_on(date(1990, 1, 1).after_days(day))
            .expect("видак продают")
    };
    let first_ratio = price(1) / price(0);
    let later_ratio = price(301) / price(300);
    assert!((first_ratio - later_ratio).abs() < 1e-12);
    let halfway = price(365);
    let geometric_mean = (90_000.0_f64 * 50_000.0).sqrt();
    assert!(
        (halfway - geometric_mean).abs() < 1e-6,
        "между 900 и 500 рублями в середине — среднее геометрическое, а не 700: {halfway}"
    );
}

#[test]
fn a_good_does_not_exist_before_its_first_point() {
    let content = content();
    let curve = &content.goods[&good("computer")].curve;
    assert_eq!(curve.kopecks_on(date(1986, 12, 31)), None);
    assert_eq!(curve.kopecks_on(date(1987, 1, 1)), Some(3_000_000.0));
    assert!(!Markets::is_on_sale(
        &content,
        &good("computer"),
        date(1986, 12, 31)
    ));
}

#[test]
fn the_denomination_strikes_three_zeros_off_the_tag() {
    let content = content();
    let rules = &content.money;
    let hundred = rubles(100);
    let before = date(1997, 1, 1);
    let eve = date(1997, 12, 31);
    let morning = date(1998, 1, 1);
    assert_eq!(
        rules.format_money(rules.from_real(hundred, Currency::Ruble, before), before),
        "1 100 000 р."
    );
    let on_eve = rules.format_money(rules.from_real(hundred, Currency::Ruble, eve), eve);
    let on_morning =
        rules.format_money(rules.from_real(hundred, Currency::Ruble, morning), morning);
    assert_eq!(on_morning, "2 000 р.");
    let digits = |text: &str| text.chars().filter(char::is_ascii_digit).count();
    assert_eq!(
        digits(&on_eve) - digits(&on_morning),
        3,
        "накануне {on_eve}"
    );
    let overnight = rules.index_on(eve) / rules.index_on(morning);
    assert!(
        (overnight - 1.0).abs() < 0.01,
        "реальная стоимость рубля за ночь деноминации не прыгает: {overnight}"
    );
}

#[test]
fn hyperinflation_eats_rubles_under_the_mattress_but_not_dollars() {
    let content = content();
    let rules = &content.money;
    let start = date(1991, 1, 1);
    let later = date(1994, 1, 1);
    let savings = rubles(1000);
    let in_rubles = rules.from_real(savings, Currency::Ruble, start);
    let in_dollars = rules.from_real(savings, Currency::Dollar, start);
    assert_eq!(rules.real_value(in_rubles, start), savings);
    assert_eq!(
        rules.real_value(in_rubles, later),
        Real::from_kopecks(113),
        "индекс вырос с 2,6 до 2300: от тысячи рублей остался рубль с копейками"
    );
    assert_eq!(
        rules.real_value(in_dollars, later),
        Real::from_kopecks(99_999),
        "доллар рос вровень с индексом"
    );
}

#[test]
fn amounts_read_like_a_dos_price_tag() {
    let content = content();
    let rules = &content.money;
    let day = date(1986, 3, 14);
    let format = |currency, minor_units| rules.format_money(Money::new(currency, minor_units), day);
    assert_eq!(format(Currency::Ruble, 124_037), "1 240 р.");
    assert_eq!(format(Currency::Ruble, 450), "4.50 р.");
    assert_eq!(format(Currency::Ruble, 300), "3 р.");
    assert_eq!(format(Currency::Ruble, 0), "0 р.");
    assert_eq!(format(Currency::Dollar, 1_840_000), "$ 18 400");
    assert_eq!(format(Currency::Checks, 8_500), "85 чек.");
    assert_eq!(
        rules.format_money(Money::new(Currency::Ruble, 5_000_000), date(1999, 1, 1)),
        "50 р.",
        "после деноминации — новые рубли"
    );
}

#[test]
fn a_city_multiplier_scales_only_its_category() {
    let content = content();
    let markets = Markets::new(&content, DRIFT_SEED);
    let day = date(1986, 3, 14);
    let price = |city_id: &str, good_id: &str| {
        markets
            .normal_kopecks(&content, &city(city_id), &good(good_id), day)
            .expect("товар продают")
    };
    assert!((price("leningrad", "jeans") / price("moscow", "jeans") - 0.95).abs() < 1e-12);
    assert_eq!(
        price("leningrad", "video_recorder"),
        price("moscow", "video_recorder")
    );
}

/// Наценка последней штуки запаса к средней цене без половины разницы цен покупки и продажи.
fn last_unit_markup(content: &Content, elasticity: f64) -> f64 {
    let jeans = good("jeans");
    let normal_stock = content.goods[&jeans].stock;
    let rules = MarketRules {
        elasticity,
        ..content.market
    };
    let normal_kopecks = 10_000.0;
    let last = CityMarket::new(&content.goods).buy_price_after(
        &jeans,
        normal_kopecks,
        normal_stock,
        &rules,
        normal_stock - 1,
    );
    last / rules.markup() / normal_kopecks
}

#[test]
fn the_elasticity_of_the_rules_softens_how_a_buyout_raises_the_price() {
    let content = content();
    let steep = last_unit_markup(&content, 1.0);
    let soft = last_unit_markup(&content, 0.5);
    assert!(
        steep > 1.5,
        "при крутизне 1 последняя штука почти вдвое дороже средней"
    );
    assert!(
        (soft - steep.sqrt()).abs() < 1e-12,
        "при крутизне 0,5 наценка — корень из наценки при 1: {soft} против {steep}"
    );
}

#[test]
fn every_next_unit_costs_more_and_a_round_trip_loses_the_spread() {
    let mut game = new_game();
    let jeans = good("jeans");
    let moscow = city("moscow");
    let tags = game
        .markets()
        .buy_tags(game.content(), &moscow, &jeans, 3, game.today())
        .expect("джинсы продают");
    assert!(tags[0].minor_units() < tags[1].minor_units());
    assert!(tags[1].minor_units() < tags[2].minor_units());
    let cash_before = game.player().money().cash();
    game.apply(buy("jeans", 3)).expect("денег хватает");
    assert_eq!(
        game.player().money().cash(),
        cash_before.minus(total(&tags))
    );
    assert_eq!(game.player().pocket().quantity(&jeans), 3);
    assert_eq!(game.markets().city(&moscow).stock(&jeans), 3);
    game.apply(sell("jeans", 3)).expect("джинсы в кармане");
    assert_eq!(game.markets().city(&moscow).stock(&jeans), 6);
    assert_eq!(game.player().pocket().quantity(&jeans), 0);
    let cash_after = game.player().money().cash();
    assert!(
        cash_before.minus(cash_after).minor_units() > 0,
        "купить и тут же продать — потерять на разнице цен"
    );
}

#[test]
fn a_big_sale_crashes_the_price_and_the_market_recovers_in_days() {
    let content = content();
    let jeans = good("jeans");
    let moscow = city("moscow");
    let mut markets = Markets::new(&content, DRIFT_SEED);
    let mut rng = WorldRng::new(7);
    let mut today = date(1986, 3, 14);
    let before = markets
        .quote(&content, &moscow, &jeans, today)
        .expect("джинсы продают");
    markets.put(&moscow, &jeans, 18);
    let crashed = markets
        .quote(&content, &moscow, &jeans, today)
        .expect("джинсы продают");
    assert!(
        crashed.sell.minor_units() * 2 < before.sell.minor_units(),
        "втрое больше обычного запаса — цена продажи ниже половины: {crashed:?} против {before:?}"
    );
    let mut yesterday = crashed;
    for _ in 0..8 {
        today = today.next();
        markets.next_day(&content, &mut rng, today);
        let recovering = markets
            .quote(&content, &moscow, &jeans, today)
            .expect("джинсы продают");
        assert!(recovering.stock < yesterday.stock, "запас тает к обычному");
        yesterday = recovering;
    }
    assert_eq!(
        markets.city(&moscow).stock(&jeans),
        6,
        "за восемь дней — обычный запас"
    );
    let untouched = Markets::new(&content, DRIFT_SEED);
    assert_eq!(
        markets.quote(&content, &moscow, &jeans, today),
        untouched.quote(&content, &moscow, &jeans, today)
    );
}

#[test]
fn a_shock_moves_a_price_for_its_term_and_a_permanent_one_stays() {
    let content = content();
    let jeans = good("jeans");
    let video_recorder = good("video_recorder");
    let moscow = city("moscow");
    let leningrad = city("leningrad");
    let mut markets = Markets::new(&content, DRIFT_SEED);
    let mut rng = WorldRng::new(7);
    let start = date(1986, 3, 14);
    let price = |markets: &Markets, city: &CityId, good: &GoodId, day: GameDate| {
        markets
            .normal_kopecks(&content, city, good, day)
            .expect("товар продают")
    };
    let calm = |city: &CityId, good: &GoodId, day: GameDate| {
        price(&Markets::new(&content, DRIFT_SEED), city, good, day)
    };
    markets.apply_shock(jeans.clone(), Some(moscow.clone()), 2.0, Some(10), start);
    assert!(
        (price(&markets, &moscow, &jeans, start) / calm(&moscow, &jeans, start) - 2.0).abs()
            < 1e-12
    );
    assert_eq!(
        price(&markets, &leningrad, &jeans, start),
        calm(&leningrad, &jeans, start)
    );
    let mut today = start;
    for _ in 0..9 {
        today = today.next();
        markets.next_day(&content, &mut rng, today);
    }
    assert!(price(&markets, &moscow, &jeans, today) > calm(&moscow, &jeans, today));
    today = today.next();
    markets.next_day(&content, &mut rng, today);
    assert_eq!(
        price(&markets, &moscow, &jeans, today),
        calm(&moscow, &jeans, today)
    );

    markets.apply_shock(video_recorder.clone(), None, 0.5, None, today);
    for _ in 0..100 {
        today = today.next();
        markets.next_day(&content, &mut rng, today);
    }
    for city in [&moscow, &leningrad] {
        let ratio =
            price(&markets, city, &video_recorder, today) / calm(city, &video_recorder, today);
        assert!((ratio - 0.5).abs() < 1e-12, "шок навсегда и везде");
    }
}

#[test]
fn money_goes_only_where_the_epoch_allows() {
    let content = content();
    let mut trader = Trader::new(Money::new(Currency::Ruble, 10_000_000), &content);
    let mut deposit_on = |id: &str, day: GameDate| trader.deposit(&content, &place(id), 10, day);
    assert!(deposit_on("savings_bank", date(1991, 12, 31)).is_ok());
    assert_eq!(
        deposit_on("savings_bank", date(1992, 1, 1)),
        Err(CommandError::PlaceClosed {
            place: "savings_bank".to_owned(),
            title: "Сберкасса".to_owned()
        })
    );
    assert!(deposit_on("commercial_bank", date(1988, 12, 31)).is_err());
    assert!(deposit_on("commercial_bank", date(1989, 1, 1)).is_ok());
    assert!(deposit_on("checks", date(1987, 12, 31)).is_ok());
    assert!(deposit_on("checks", date(1988, 1, 1)).is_err());
    let dollars = &content.money_places[&place("dollars")];
    assert!(
        !dollars.is_legal_on(date(1990, 12, 31)),
        "до 1991 года — статья"
    );
    assert!(dollars.is_legal_on(date(1991, 1, 1)));
}

#[test]
fn dollars_are_bought_and_sold_for_cash_at_the_rate_of_the_day() {
    let content = content();
    let dollars = place("dollars");
    let day = date(1984, 1, 1);
    let mut trader = Trader::new(Money::new(Currency::Ruble, 100_000), &content);
    let bought = trader
        .deposit(&content, &dollars, 100, day)
        .expect("на 300 рублей денег хватает");
    assert_eq!(bought, Money::new(Currency::Dollar, 10_000));
    assert_eq!(
        trader.money().cash(),
        Money::new(Currency::Ruble, 70_000),
        "доллар по 3 рубля"
    );
    trader
        .withdraw(&content, &dollars, 50, day)
        .expect("сто долларов лежат");
    assert_eq!(
        trader.money().balance(&dollars),
        Money::new(Currency::Dollar, 5_000)
    );
    assert_eq!(trader.money().cash(), Money::new(Currency::Ruble, 85_000));
    assert_eq!(
        trader.money().real_total(&content.money, day),
        rubles(1000),
        "обмен по курсу дня ничего не съедает"
    );
}

#[test]
fn a_savings_bank_pays_interest_on_the_first_of_the_month() {
    let content = content();
    let savings_bank = place("savings_bank");
    let mut trader = Trader::new(Money::new(Currency::Ruble, 200_000), &content);
    trader
        .deposit(&content, &savings_bank, 1000, date(1984, 1, 1))
        .expect("сберкасса открыта");
    trader.next_day(&content, date(1984, 1, 31));
    assert_eq!(
        trader.money().balance(&savings_bank),
        Money::new(Currency::Ruble, 100_000)
    );
    trader.next_day(&content, date(1984, 2, 1));
    assert_eq!(
        trader.money().balance(&savings_bank),
        Money::new(Currency::Ruble, 100_167),
        "2% годовых — шестая часть процента в месяц"
    );
    assert_eq!(
        trader.money().balance(&place("mattress")),
        Money::zero(Currency::Ruble)
    );
}

#[test]
fn the_snapshot_shows_the_market_pocket_and_money_from_the_core() {
    let mut game = new_game();
    let view = game.view();
    assert_eq!(view.header.city, "Москва");
    assert_eq!(view.header.money, vec!["Наличные: 2 000 р."]);
    assert_eq!(view.market.place, "Толкучка у Рижского вокзала");
    let rows: Vec<(&str, &str, &str, &str)> = view
        .market
        .goods
        .iter()
        .map(|row| {
            (
                row.good.as_str(),
                row.buy.as_str(),
                row.sell.as_str(),
                row.stock.as_str(),
            )
        })
        .collect();
    assert_eq!(
        rows,
        vec![
            ("jeans", "197", "164", "6"),
            ("video_recorder", "1 440", "1 013", "2"),
            ("vodka_bottle", "5.64", "4.97", "20"),
        ],
        "компьютера в 1984 году ещё нет"
    );

    game.apply(buy("jeans", 2)).expect("денег хватает");
    game.apply(deposit("dollars", 100))
        .expect("доллары у фарцовщика");
    let view = game.view();
    assert_eq!(
        view.header.money,
        vec!["Наличные: 1 287 р.", "$ 100", "Курс: 3 р."]
    );
    assert_eq!(view.market.goods[0].stock, "4");
    assert_eq!(
        view.pocket.items,
        vec![
            PocketItemView {
                label: "Джинсы".to_owned(),
                amount: "2 шт".to_owned()
            },
            PocketItemView {
                label: "Доллары".to_owned(),
                amount: "$ 100".to_owned()
            },
            PocketItemView {
                label: "Связей".to_owned(),
                amount: "0".to_owned()
            },
        ]
    );
    assert_eq!(
        view.messages,
        vec![
            "Купил: Джинсы (фирма), 2 шт за 413 р.",
            "Доллары: положил $ 100"
        ]
    );
    let titles: Vec<&str> = view
        .money
        .places
        .iter()
        .map(|spot| spot.title.as_str())
        .collect();
    assert_eq!(
        titles,
        vec!["Чеки «Берёзки»", "Доллары", "Под матрасом", "Сберкасса"],
        "коммерческих банков в 1984 году нет"
    );
    assert_eq!(
        view.money.places[1],
        MoneyPlaceView {
            place: "dollars".to_owned(),
            title: "Доллары".to_owned(),
            amount: "$ 100".to_owned(),
            risk: "Статья до 90-х, фальшивки, «кидалы» в обменниках".to_owned()
        }
    );
    assert_eq!(view.money.rates, vec!["Чек ВПТ: 2 р.", "Доллар: 3 р."]);
}

#[test]
fn refused_deals_name_the_reason_and_change_nothing() {
    let mut game = new_game();
    let before = serde_json::to_string(&game.view()).expect("снимок сериализуется");
    let refusals = [
        (
            buy("jeans", 7),
            CommandError::OutOfStock {
                good: "jeans".to_owned(),
                name: "Джинсы (фирма)".to_owned(),
                available: 6,
            },
        ),
        (
            buy("computer", 1),
            CommandError::NotOnSale {
                good: "computer".to_owned(),
                name: "Компьютер".to_owned(),
            },
        ),
        (
            buy("video_recorder", 2),
            CommandError::NotEnoughCash {
                needed: "3 456 р.".to_owned(),
                cash: "2 000 р.".to_owned(),
            },
        ),
        (
            sell("jeans", 1),
            CommandError::NotInPocket {
                good: "jeans".to_owned(),
                name: "Джинсы (фирма)".to_owned(),
                held: 0,
            },
        ),
        (
            buy("caviar", 1),
            CommandError::UnknownGood {
                good: "caviar".to_owned(),
            },
        ),
        (buy("jeans", 0), CommandError::ZeroQuantity),
        (deposit("mattress", 0), CommandError::ZeroAmount),
        (
            deposit("swiss_bank", 10),
            CommandError::UnknownPlace {
                place: "swiss_bank".to_owned(),
            },
        ),
        (
            withdraw("mattress", 10),
            CommandError::NotEnoughInPlace {
                place: "mattress".to_owned(),
                title: "Под матрасом".to_owned(),
                balance: "0 р.".to_owned(),
            },
        ),
    ];
    for (command, refusal) in refusals {
        assert_eq!(game.apply(command), Err(refusal));
    }
    assert_eq!(
        serde_json::to_string(&game.view()).expect("снимок сериализуется"),
        before
    );
    assert_eq!(
        CommandError::NotEnoughCash {
            needed: "3 456 р.".to_owned(),
            cash: "2 000 р.".to_owned()
        }
        .to_string(),
        "не хватает денег: нужно 3 456 р., при себе 2 000 р."
    );
}

#[test]
fn deposits_and_withdrawals_are_logged_in_the_messages() {
    let mut game = new_game();
    game.apply(deposit("savings_bank", 500))
        .expect("сберкасса открыта");
    game.apply(withdraw("savings_bank", 100))
        .expect("вклад есть");
    let view = game.view();
    assert_eq!(view.header.money, vec!["Наличные: 1 600 р."]);
    assert_eq!(
        view.messages,
        vec!["Сберкасса: положил 500 р.", "Сберкасса: снял 100 р."]
    );
}

#[test]
fn an_unknown_biography_does_not_start_a_life() {
    let settings = LifeSettings {
        biography: "cosmonaut".to_owned(),
        ironman: false,
        seed: 1,
    };
    assert_eq!(
        Game::new_life(&settings, content()).err(),
        Some(CommandError::UnknownBiography {
            biography: "cosmonaut".to_owned()
        })
    );
}
