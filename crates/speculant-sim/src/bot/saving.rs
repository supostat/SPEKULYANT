use speculant_core::content::MoneyPlaceId;
use speculant_core::economy::Currency;
use speculant_core::time::GameDate;
use speculant_core::{Command, Content, Game};

/// Какое из мест, что платят процент, бот выбирает для вклада.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PlacePreference {
    /// Самый скромный процент — самое скучное и надёжное место.
    LowestInterest,
    /// Самый жирный процент, чем бы он ни кончился.
    HighestInterest,
}

/// Вклады: раз в месяц бот держит в выбранном месте долю своих денег — наличных и вклада, — а из
/// прочих мест забирает всё; под матрас не кладёт.
pub(crate) struct Saving {
    pub share: f64,
    pub preference: PlacePreference,
}

impl Saving {
    pub fn rebalance(&self, game: &mut Game) {
        let chosen = self.place(game.content(), game.today());
        let today = game.today();
        let unit = game
            .content()
            .money
            .whole_units(Currency::Ruble, 1, today)
            .minor_units();
        let elsewhere: Vec<(MoneyPlaceId, i64)> = ruble_places(game.content())
            .filter(|place| Some(place) != chosen.as_ref())
            .map(|place| {
                let units = game.player().money().balance(&place).minor_units() / unit;
                (place, units)
            })
            .filter(|(_, units)| *units > 0)
            .collect();
        for (place, units) in elsewhere {
            move_money(game, place, -units);
        }
        let Some(place) = chosen else {
            return;
        };
        let cash = game.player().money().cash().minor_units();
        let deposit = game.player().money().balance(&place).minor_units();
        let target = ((cash + deposit) as f64 * self.share) as i64;
        let units = (target - deposit) / unit;
        if units != 0 {
            move_money(game, place, units);
        }
    }

    /// Место для вклада сегодня: рублёвое, не дома, принимает вклады, законное и платит процент.
    fn place(&self, content: &Content, today: GameDate) -> Option<MoneyPlaceId> {
        let paying = ruble_places(content).filter_map(|place| {
            let spot = &content.money_places[&place];
            let percent = spot.annual_percent_on(today);
            (spot.accepts_deposits_on(today) && spot.is_legal_on(today) && percent > 0.0)
                .then_some((place, percent))
        });
        let chosen = match self.preference {
            PlacePreference::LowestInterest => {
                paying.min_by(|left, right| left.1.total_cmp(&right.1))
            }
            PlacePreference::HighestInterest => {
                paying.max_by(|left, right| left.1.total_cmp(&right.1))
            }
        };
        chosen.map(|(place, _)| place)
    }
}

/// Рублёвые места хранения не дома — те, где бот держит вклады.
fn ruble_places(content: &Content) -> impl Iterator<Item = MoneyPlaceId> + '_ {
    content
        .money_places
        .iter()
        .filter(|(_, place)| place.currency == Currency::Ruble && !place.home)
        .map(|(id, _)| id.clone())
}

/// Кладёт `units` целых рублей, если их больше нуля, или забирает, если меньше.
fn move_money(game: &mut Game, place: MoneyPlaceId, units: i64) {
    let amount = units.unsigned_abs();
    let command = if units > 0 {
        Command::Deposit {
            place: place.to_string(),
            amount,
        }
    } else {
        Command::Withdraw {
            place: place.to_string(),
            amount,
        }
    };
    game.apply(command)
        .expect("бот кладёт из наличных и забирает не больше вклада");
}

#[cfg(test)]
mod tests {
    use speculant_core::content::MoneyPlaceId;

    use super::{PlacePreference, Saving};
    use crate::test_worlds::{date, new_life, real_data};

    fn saving(preference: PlacePreference) -> Saving {
        Saving {
            share: 0.5,
            preference,
        }
    }

    fn place(id: &str) -> Option<MoneyPlaceId> {
        Some(MoneyPlaceId::new(id))
    }

    #[test]
    fn the_cautious_keep_to_the_smallest_interest_and_the_reckless_chase_the_biggest() {
        let content = real_data();
        let lowest = saving(PlacePreference::LowestInterest);
        let highest = saving(PlacePreference::HighestInterest);
        let soviet = date(1985, 3, 1);
        assert_eq!(lowest.place(&content, soviet), place("savings_bank"));
        assert_eq!(highest.place(&content, soviet), place("savings_bank"));
        let wild = date(1993, 6, 1);
        assert_eq!(lowest.place(&content, wild), place("commercial_bank"));
        assert_eq!(highest.place(&content, wild), place("pyramids"));
        let late = date(1996, 6, 1);
        assert_eq!(
            highest.place(&content, late),
            place("gko"),
            "пирамиды рухнули"
        );
    }

    #[test]
    fn a_monthly_rebalance_keeps_the_share_of_money_in_the_chosen_place() {
        let mut game = new_life(&real_data(), "student");
        let cash = game.player().money().cash().minor_units();
        saving(PlacePreference::LowestInterest).rebalance(&mut game);
        let deposit = game
            .player()
            .money()
            .balance(&MoneyPlaceId::new("savings_bank"))
            .minor_units();
        assert_eq!(deposit, cash / 2);
        assert_eq!(game.player().money().cash().minor_units(), cash - deposit);
    }
}
