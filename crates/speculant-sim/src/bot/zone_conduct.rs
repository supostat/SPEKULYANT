use speculant_core::economy::Currency;
use speculant_core::{Command, Game};

/// Как бот сидит: все работают на промзоне и просятся по УДО, как только срок позволяет; кто
/// откупается — платит, если зона эпохи продаёт волю и наличных хватает.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ZoneConduct {
    Patient,
    Briber,
}

impl ZoneConduct {
    pub fn serve_week(self, game: &mut Game) {
        let working = game
            .zone()
            .expect("неделю в зоне живёт сидящий")
            .is_working();
        if !working {
            game.apply(Command::SetWork { work: true })
                .expect("работать в зоне можно всегда");
        }
        if self == Self::Briber && can_bribe(game) {
            game.apply(Command::BribeForRelease)
                .expect("зона продаёт волю, наличных хватает");
            return;
        }
        let today = game.today();
        let parole_from = game
            .zone()
            .expect("ещё в зоне")
            .parole_from(game.content().camp_on(today));
        if today >= parole_from {
            game.apply(Command::ApplyForParole)
                .expect("срок для просьбы об УДО пришёл");
        }
    }
}

fn can_bribe(game: &Game) -> bool {
    let today = game.today();
    game.content().camp_on(today).bribe.is_some_and(|bribe| {
        let price = game
            .content()
            .money
            .from_real(bribe, Currency::Ruble, today);
        game.player().money().cash().covers(price)
    })
}

#[cfg(test)]
mod tests {
    use speculant_core::content::EventId;
    use speculant_core::{Command, Game};

    use super::ZoneConduct;
    use crate::test_worlds::{core_fixture, date, live_until, new_life};

    /// Суд даёт год: признание в событии ареста фикстуры зоны.
    fn sentenced() -> Game {
        let mut game = new_life(&core_fixture("zone"), "student");
        sentence(&mut game);
        game
    }

    fn sentence(game: &mut Game) {
        game.start_event(&EventId::new("arrest"))
            .expect("арест готов");
        game.apply(Command::Choose { choice: 1 })
            .expect("признание");
        game.apply(Command::Continue).expect("исход прочитан");
        assert!(game.zone().is_some(), "игрок сел");
    }

    #[test]
    fn a_patient_convict_works_and_walks_out_on_parole_as_soon_as_the_term_allows() {
        let mut game = sentenced();
        let day = game.content().time.clock.day_milliseconds_at(1);
        ZoneConduct::Patient.serve_week(&mut game);
        let zone = game.zone().expect("ещё сидит");
        assert!(zone.is_working());
        let parole_from = zone.parole_from(game.content().camp_on(game.today()));
        while game.zone().is_some() {
            ZoneConduct::Patient.serve_week(&mut game);
            if game.zone().is_some() {
                game.advance(day);
            }
        }
        assert!(game.today() >= parole_from);
        assert!(
            game.today() < parole_from.after_days(7),
            "попросился в первую же неделю, а УДО в фикстуре дают всегда"
        );
    }

    #[test]
    fn a_briber_buys_his_way_out_where_the_zone_sells_freedom() {
        let mut soviet = sentenced();
        ZoneConduct::Briber.serve_week(&mut soviet);
        assert!(soviet.zone().is_some(), "советская зона волю не продаёт");

        let mut game = new_life(&core_fixture("zone"), "student");
        live_until(&mut game, date(1991, 12, 20));
        sentence(&mut game);
        live_until(&mut game, date(1992, 1, 1));
        ZoneConduct::Briber.serve_week(&mut game);
        assert_eq!(game.zone(), None, "зона девяностых отпустила за взятку");
    }
}
