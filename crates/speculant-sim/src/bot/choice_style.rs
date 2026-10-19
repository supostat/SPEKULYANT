use rand_chacha::ChaCha8Rng;
use speculant_core::Game;
use speculant_core::content::Consequences;
use speculant_core::view::EventView;

use super::dice;

/// Как бот выбирает в событии. Бот видит последствия выбора из данных, но не видит продолжений:
/// они разыгрываются после.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChoiceStyle {
    /// Любой доступный.
    Random,
    /// Наименее опасный, а из равных — самый денежный.
    Safest,
    /// Самый денежный, а из равных — наименее опасный.
    Greediest,
    /// Самый денежный из тех, что не кончают жизнь и не дают срока; если таких нет — наименее
    /// опасный.
    Sensible,
}

/// Опасность выбора по убыванию тяжести: конец жизни, месяцы срока, потерянное здоровье.
type Danger = (bool, u32, u32);

impl ChoiceStyle {
    /// Номер выбора от 1 среди доступных.
    pub fn choose(self, game: &Game, event: &EventView, dice: &mut ChaCha8Rng) -> usize {
        let pending = game
            .events()
            .pending()
            .expect("бот выбирает в ожидающем событии");
        let choices = &game.content().events[&pending.event].choices;
        let available: Vec<(usize, &Consequences)> = event
            .choices
            .iter()
            .zip(choices)
            .enumerate()
            .filter(|(_, (shown, _))| shown.available)
            .map(|(index, (_, choice))| (index, &choice.consequences))
            .collect();
        assert!(
            !available.is_empty(),
            "у события {} нет доступного выбора",
            pending.event
        );
        self.pick(game, &available, dice) + 1
    }

    /// Номер от нуля среди доступных выборов — пар «номер, последствия».
    fn pick(
        self,
        game: &Game,
        available: &[(usize, &Consequences)],
        dice: &mut ChaCha8Rng,
    ) -> usize {
        match self {
            Self::Random => available[dice::index(dice, available.len())].0,
            Self::Safest => safest(game, available),
            Self::Greediest => {
                let (index, _) = available
                    .iter()
                    .max_by(|left, right| {
                        gain(game, left.1)
                            .total_cmp(&gain(game, right.1))
                            .then_with(|| danger(right.1).cmp(&danger(left.1)))
                    })
                    .expect("доступный выбор есть");
                *index
            }
            Self::Sensible => {
                let harmless: Vec<(usize, &Consequences)> = available
                    .iter()
                    .copied()
                    .filter(|(_, consequences)| {
                        let (ends_life, months, _) = danger(consequences);
                        !ends_life && months == 0
                    })
                    .collect();
                harmless
                    .iter()
                    .max_by(|left, right| gain(game, left.1).total_cmp(&gain(game, right.1)))
                    .map_or_else(|| safest(game, available), |(index, _)| *index)
            }
        }
    }
}

fn safest(game: &Game, available: &[(usize, &Consequences)]) -> usize {
    let (index, _) = available
        .iter()
        .min_by(|left, right| {
            danger(left.1)
                .cmp(&danger(right.1))
                .then_with(|| gain(game, right.1).total_cmp(&gain(game, left.1)))
        })
        .expect("доступный выбор есть");
    *index
}

fn danger(consequences: &Consequences) -> Danger {
    let longer_term = consequences
        .zone
        .as_ref()
        .map_or(0, |zone| zone.term_months.max(0).unsigned_abs());
    (
        consequences.end_life.is_some(),
        consequences.sentence_months.unwrap_or(0) + longer_term,
        consequences.health.min(0).unsigned_abs(),
    )
}

/// Выгода выбора в копейках 1984 года: деньги и товар по средней цене города.
fn gain(game: &Game, consequences: &Consequences) -> f64 {
    let money = consequences.money.map_or(0, |money| money.kopecks()) as f64;
    let goods: f64 = consequences
        .goods
        .iter()
        .filter_map(|change| {
            game.markets()
                .normal_kopecks(game.content(), game.city(), &change.good, game.today())
                .map(|price| price * f64::from(change.change))
        })
        .sum();
    money + goods
}

#[cfg(test)]
mod tests {
    use rand_chacha::ChaCha8Rng;
    use speculant_core::content::{Consequences, EndingId};
    use speculant_core::economy::Real;
    use speculant_core::rng::WorldRng;

    use super::ChoiceStyle;
    use crate::test_worlds::{new_life, real_data};

    fn money(rubles: i64) -> Consequences {
        Consequences {
            money: Some(Real::from_kopecks(rubles * 100)),
            ..Consequences::default()
        }
    }

    /// Выборы события: 0 — много денег и год срока, 1 — немного денег, 2 — убыток и подорванное
    /// здоровье, 3 — деньги и смерть.
    fn choices() -> Vec<Consequences> {
        vec![
            Consequences {
                sentence_months: Some(12),
                ..money(500)
            },
            money(100),
            Consequences {
                health: -10,
                ..money(-50)
            },
            Consequences {
                end_life: Some(EndingId::new("death_bandits")),
                ..money(1000)
            },
        ]
    }

    fn pick(style: ChoiceStyle, among: &[usize], dice: &mut ChaCha8Rng) -> usize {
        let game = new_life(&real_data(), "student");
        let choices = choices();
        let available: Vec<_> = among
            .iter()
            .map(|index| (*index, &choices[*index]))
            .collect();
        style.pick(&game, &available, dice)
    }

    #[test]
    fn styles_weigh_money_against_term_health_and_death() {
        let mut world = WorldRng::new(1);
        let dice = world.stream("bot");
        assert_eq!(pick(ChoiceStyle::Safest, &[0, 1, 2, 3], dice), 1);
        assert_eq!(pick(ChoiceStyle::Greediest, &[0, 1, 2, 3], dice), 3);
        assert_eq!(pick(ChoiceStyle::Greediest, &[0, 1, 2], dice), 0);
        assert_eq!(
            pick(ChoiceStyle::Sensible, &[0, 1, 2, 3], dice),
            1,
            "без срока и смерти — самый денежный"
        );
        assert_eq!(
            pick(ChoiceStyle::Sensible, &[0, 3], dice),
            0,
            "все выборы опасны — наименее опасный"
        );
    }

    #[test]
    fn the_random_style_takes_every_available_choice_and_nothing_else() {
        let mut world = WorldRng::new(1);
        let dice = world.stream("bot");
        let mut taken: Vec<usize> = (0..60)
            .map(|_| pick(ChoiceStyle::Random, &[1, 3], dice))
            .collect();
        taken.sort_unstable();
        taken.dedup();
        assert_eq!(taken, vec![1, 3]);
    }
}
