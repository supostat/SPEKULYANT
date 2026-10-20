//! Экономика мира (§5, §6). Цены ядро считает в реальных единицах — копейках 1984 года; номинал на
//! ценнике, деноминация и валюты — слой поверх через индекс инфляции (§5.1). Деньги игрока лежат в
//! своей валюте, поэтому гиперинфляция бьёт ровно по тем, кто держит рубли.

mod city_market;
mod currency;
mod denomination;
mod log_interpolation;
mod market_rules;
mod market_swings;
mod markets;
mod money;
mod money_rules;
mod money_store;
mod pocket;
mod price_curve;
mod price_drift;
mod price_history;
mod real;
mod ruble_curve;
mod seen_prices;
mod shock;
mod trader;

pub use city_market::CityMarket;
pub use currency::Currency;
pub use denomination::Denomination;
pub use market_rules::MarketRules;
pub use market_swings::{FactorRange, Swing, SwingRules};
pub use markets::{Markets, Quote};
pub use money::Money;
pub use money_rules::MoneyRules;
pub use money_store::MoneyStore;
pub use pocket::Pocket;
pub use price_curve::{PriceCurve, PricePoint};
pub use price_drift::{DriftRules, PriceDrift};
pub use price_history::{HISTORY_WEEKS, PriceHistory, PriceWeek, WEEK_DAYS};
pub use real::Real;
pub use ruble_curve::{RubleCurve, RublePoint};
pub use seen_prices::{SeenMarket, SeenPrice, SeenPrices};
pub use shock::Shock;
pub use trader::{Deal, Trader};
