use std::fmt;

use serde::Serialize;

use crate::bot::{Arbitrage, Bot, ChoiceStyle, Hoarding, PlacePreference, RandomMoves, Saving};
use crate::bot::{Trade, ZoneConduct};

/// Стратегия бота (§14.1): как он торгует, выбирает в событиях, копит деньги и сидит.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Делает что попало: точка отсчёта, ниже которой играть нельзя.
    Random,
    /// Возит товар туда, где он дороже.
    Arbitrage,
    /// Знает историю цен и держит товар до пика.
    Hoarder,
    /// Возит товар, чтобы открывать дела и нанимать управляющих.
    Entrepreneur,
    /// Ездит недалеко, ставит половину, держит деньги в самом скучном месте и не рискует.
    Cautious,
    /// Ставит всё, хватает самый жирный выбор, несёт деньги туда, где процент выше, и откупается.
    Reckless,
}

impl Strategy {
    pub const ALL: [Self; 6] = [
        Self::Random,
        Self::Arbitrage,
        Self::Hoarder,
        Self::Entrepreneur,
        Self::Cautious,
        Self::Reckless,
    ];

    pub fn id(self) -> &'static str {
        match self {
            Self::Random => "random",
            Self::Arbitrage => "arbitrage",
            Self::Hoarder => "hoarder",
            Self::Entrepreneur => "entrepreneur",
            Self::Cautious => "cautious",
            Self::Reckless => "reckless",
        }
    }

    pub fn parse(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|strategy| strategy.id() == id)
    }

    /// Бот стратегии, собранный из манеры торговать, выбирать, копить и сидеть.
    pub(crate) fn bot(self) -> Bot {
        match self {
            Self::Random => Bot {
                trade: Trade::Random(RandomMoves { act_chance: 0.3 }),
                choices: ChoiceStyle::Random,
                saving: None,
                enterprise: false,
                zone: ZoneConduct::Patient,
            },
            Self::Arbitrage => Bot {
                trade: Trade::Arbitrage(Arbitrage {
                    stake: 0.9,
                    longest_route_days: u32::MAX,
                    idle_until: None,
                }),
                choices: ChoiceStyle::Sensible,
                saving: None,
                enterprise: false,
                zone: ZoneConduct::Patient,
            },
            Self::Hoarder => Bot {
                trade: Trade::Hoard(Hoarding {
                    stake: 0.8,
                    outlook_days: 730,
                    least_growth: 1.3,
                }),
                choices: ChoiceStyle::Sensible,
                saving: None,
                enterprise: false,
                zone: ZoneConduct::Patient,
            },
            Self::Entrepreneur => Bot {
                trade: Trade::Arbitrage(Arbitrage {
                    stake: 0.8,
                    longest_route_days: u32::MAX,
                    idle_until: None,
                }),
                choices: ChoiceStyle::Sensible,
                saving: None,
                enterprise: true,
                zone: ZoneConduct::Patient,
            },
            Self::Cautious => Bot {
                trade: Trade::Arbitrage(Arbitrage {
                    stake: 0.5,
                    longest_route_days: 1,
                    idle_until: None,
                }),
                choices: ChoiceStyle::Safest,
                saving: Some(Saving {
                    share: 0.5,
                    preference: PlacePreference::LowestInterest,
                }),
                enterprise: false,
                zone: ZoneConduct::Patient,
            },
            Self::Reckless => Bot {
                trade: Trade::Arbitrage(Arbitrage {
                    stake: 1.0,
                    longest_route_days: u32::MAX,
                    idle_until: None,
                }),
                choices: ChoiceStyle::Greediest,
                saving: Some(Saving {
                    share: 0.3,
                    preference: PlacePreference::HighestInterest,
                }),
                enterprise: false,
                zone: ZoneConduct::Briber,
            },
        }
    }
}

impl fmt::Display for Strategy {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.id())
    }
}
