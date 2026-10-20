//! Выгода торговли: лучший рейс с кошельком раз в неделю каждой эпохи — по открытым маршрутам из
//! открытых городов, при обычном запасе и дрейфе цен по сиду, за вычетом проезда; выгода — прибыль
//! к кошельку.

use std::path::Path;

use speculant_core::Content;
use speculant_core::content::{CategoryId, CityId};
use speculant_core::economy::Real;
use speculant_sim::{BalanceFile, TradeReport};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
const TRAVEL: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../speculant-core/tests/fixtures/travel"
);
const EPOCHS: [&str; 4] = ["stagnation", "perestroika", "wild_market", "stabilization"];
/// Недель в застое: с 1 января 1984 года по 31 декабря 1987-го.
const STAGNATION_WEEKS: usize = 209;
/// Дрейф цен в фикстуре поездок выключен: сид ни на что не влияет.
const DRIFT_SEED: u64 = 1;

#[test]
fn on_the_real_data_the_best_trip_of_every_epoch_pays_its_fare() {
    let content = Content::load(Path::new(DATA)).expect("data/ валиден");
    let rules = BalanceFile::read(Path::new(DATA)).expect("data/balance.toml разбирается");
    let trade = TradeReport::build(&content, rules.trade.purse, rules.balance.seed);
    let epochs: Vec<&str> = trade
        .epochs
        .iter()
        .map(|epoch| epoch.epoch.as_str())
        .collect();
    assert_eq!(epochs, EPOCHS, "эпохи, у которых есть конец");
    assert_eq!(trade.epochs[0].weeks, STAGNATION_WEEKS);
    for epoch in &trade.epochs {
        assert!(
            epoch.trip_return > 0.0,
            "{}: лучший рейс окупает дорогу, а не {:.1}%",
            epoch.epoch,
            epoch.trip_return * 100.0
        );
    }
}

#[test]
fn a_trip_from_a_city_that_is_still_closed_does_not_count() {
    let mut content = Content::load(Path::new(TRAVEL)).expect("фикстура поездок валидна");
    content
        .cities
        .get_mut(&CityId::new("warsaw"))
        .expect("Варшава в фикстуре")
        .multipliers
        .insert(CategoryId::new("clothes"), 0.3);
    let trade = TradeReport::build(&content, Real::from_kopecks(50_000), DRIFT_SEED);
    let stagnation = trade.epoch("stagnation").expect("застой с концом");
    assert!(
        stagnation.trip_return < 0.0,
        "Варшава откроется только в 1990-м: дешёвые джинсы оттуда в застой не возят, а из Москвы и \
         Ленинграда везти нечего — {:.1}%",
        stagnation.trip_return * 100.0
    );
}
