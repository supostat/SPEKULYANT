use rand_chacha::ChaCha8Rng;
use speculant_core::Game;
use speculant_core::content::Choice;
use speculant_core::view::EventView;

use super::choice_style::available_choices;
use super::dice;

/// Любопытство бота в прогоне покрытия: в доле `share` решений он берёт наугад любой доступный
/// выбор, кроме того, что кончает жизнь, — так прогон заглядывает в продолжения, которые манера
/// бота сама никогда не выберет: вложения, отказы, долги. В замере баланса доля — ноль.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Curiosity {
    pub share: f64,
}

impl Curiosity {
    /// Номер выбора от 1, если на этот раз бот любопытен и ему есть что выбрать, не кончая жизнь.
    pub fn pick(self, game: &Game, event: &EventView, dice: &mut ChaCha8Rng) -> Option<usize> {
        self.pick_among(&available_choices(game, event), dice)
            .map(|index| index + 1)
    }

    /// Номер от нуля среди доступных выборов — пар «номер, выбор».
    fn pick_among(self, available: &[(usize, &Choice)], dice: &mut ChaCha8Rng) -> Option<usize> {
        if !dice::chance(dice, self.share) {
            return None;
        }
        let living: Vec<usize> = available
            .iter()
            .filter(|(_, choice)| choice.consequences.end_life.is_none())
            .map(|(index, _)| *index)
            .collect();
        (!living.is_empty()).then(|| living[dice::index(dice, living.len())])
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use speculant_core::content::{Choice, Consequences, EndingId};
    use speculant_core::rng::WorldRng;

    use super::Curiosity;

    fn choice(end_life: Option<&str>) -> Choice {
        Choice {
            text: "Выбор".to_owned(),
            conditions: Default::default(),
            consequences: Consequences {
                end_life: end_life.map(EndingId::new),
                ..Consequences::default()
            },
            outcome: Some("Исход.".to_owned()),
            follow_ups: Vec::new(),
        }
    }

    #[test]
    fn a_curious_bot_tries_every_choice_that_keeps_it_alive() {
        let choices = [choice(None), choice(Some("emigration")), choice(None)];
        let available: Vec<_> = choices.iter().enumerate().collect();
        let mut world = WorldRng::new(1);
        let dice = world.stream("curiosity");
        let always = Curiosity { share: 1.0 };
        let picked: BTreeSet<usize> = (0..50)
            .map(|_| {
                always
                    .pick_among(&available, dice)
                    .expect("любопытен всегда")
            })
            .collect();
        assert_eq!(picked, BTreeSet::from([0, 2]));
    }

    #[test]
    fn without_curiosity_or_a_living_choice_the_bot_keeps_its_manner() {
        let choices = [choice(None), choice(Some("death_bandits"))];
        let available: Vec<_> = choices.iter().enumerate().collect();
        let mut world = WorldRng::new(1);
        let dice = world.stream("curiosity");
        let never = Curiosity { share: 0.0 };
        assert!((0..50).all(|_| never.pick_among(&available, dice).is_none()));
        let always = Curiosity { share: 1.0 };
        assert_eq!(always.pick_among(&available[1..], dice), None);
    }
}
