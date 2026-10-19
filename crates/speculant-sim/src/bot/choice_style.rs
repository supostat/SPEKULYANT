use rand_chacha::ChaCha8Rng;
use speculant_core::Game;
use speculant_core::content::{Choice, Consequences};
use speculant_core::life::EndingKind;
use speculant_core::view::EventView;

use super::dice;

/// Как бот выбирает в событии. Бот видит последствия выбора из данных и то, что выбор назначает
/// продолжение, но не видит, каким оно будет: продолжение для него — неизвестность.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum ChoiceStyle {
    /// Любой доступный.
    Random,
    /// Наименее опасный, а из равных — самый денежный.
    Safest,
    /// Самый денежный, а из равных — наименее опасный.
    Greediest,
    /// Самый денежный из тех, что не кончают жизнь и не дают срока, а из равных — наименее
    /// опасный; если таких нет — наименее опасный.
    Sensible,
}

/// Опасность выбора по убыванию тяжести: беда (финал не по своей воле), любой конец жизни —
/// эмиграция тоже, месяцы срока, потерянное здоровье, неизвестное продолжение.
type Danger = (bool, bool, u32, u32, bool);

impl ChoiceStyle {
    /// Номер выбора от 1 среди доступных.
    pub fn choose(self, game: &Game, event: &EventView, dice: &mut ChaCha8Rng) -> usize {
        self.pick(game, &available_choices(game, event), dice) + 1
    }

    /// Номер от нуля среди доступных выборов — пар «номер, выбор».
    fn pick(self, game: &Game, available: &[(usize, &Choice)], dice: &mut ChaCha8Rng) -> usize {
        match self {
            Self::Random => available[dice::index(dice, available.len())].0,
            Self::Safest => safest(game, available),
            Self::Greediest => greediest(game, available).expect("доступный выбор есть"),
            Self::Sensible => {
                let harmless: Vec<(usize, &Choice)> = available
                    .iter()
                    .copied()
                    .filter(|(_, choice)| {
                        let (_, ends_life, months, _, _) = danger(game, choice);
                        !ends_life && months == 0
                    })
                    .collect();
                greediest(game, &harmless).unwrap_or_else(|| safest(game, available))
            }
        }
    }
}

/// Доступные выборы ожидающего события — пары «номер от нуля, выбор из данных»; хотя бы один есть
/// всегда: это обещает правило выбора, доступного всегда.
pub(crate) fn available_choices<'a>(game: &'a Game, event: &EventView) -> Vec<(usize, &'a Choice)> {
    let pending = game
        .events()
        .pending()
        .expect("бот выбирает в ожидающем событии");
    let choices = &game.content().events[&pending.event].choices;
    let available: Vec<(usize, &Choice)> = event
        .choices
        .iter()
        .zip(choices)
        .enumerate()
        .filter(|(_, (shown, _))| shown.available)
        .map(|(index, (_, choice))| (index, choice))
        .collect();
    assert!(
        !available.is_empty(),
        "у события {} нет доступного выбора",
        pending.event
    );
    available
}

/// Самый денежный, а из равных — наименее опасный; `None`, если выбирать не из чего.
fn greediest(game: &Game, available: &[(usize, &Choice)]) -> Option<usize> {
    available
        .iter()
        .max_by(|left, right| {
            gain(game, &left.1.consequences)
                .total_cmp(&gain(game, &right.1.consequences))
                .then_with(|| danger(game, right.1).cmp(&danger(game, left.1)))
        })
        .map(|(index, _)| *index)
}

fn safest(game: &Game, available: &[(usize, &Choice)]) -> usize {
    let (index, _) = available
        .iter()
        .min_by(|left, right| {
            danger(game, left.1)
                .cmp(&danger(game, right.1))
                .then_with(|| {
                    gain(game, &right.1.consequences).total_cmp(&gain(game, &left.1.consequences))
                })
        })
        .expect("доступный выбор есть");
    *index
}

fn danger(game: &Game, choice: &Choice) -> Danger {
    let consequences = &choice.consequences;
    let longer_term = consequences
        .zone
        .as_ref()
        .map_or(0, |zone| zone.term_months.max(0).unsigned_abs());
    let loss = consequences
        .end_life
        .as_ref()
        .is_some_and(|ending| game.content().endings[ending].kind != EndingKind::Departure);
    (
        loss,
        consequences.end_life.is_some(),
        consequences.sentence_months.unwrap_or(0) + longer_term,
        consequences.health.min(0).unsigned_abs(),
        !choice.follow_ups.is_empty(),
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
    use speculant_core::content::{Choice, Consequences, EndingId, EventId, FollowUp};
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

    fn choice(consequences: Consequences) -> Choice {
        Choice {
            text: "Выбор".to_owned(),
            conditions: Default::default(),
            consequences,
            outcome: Some("Исход.".to_owned()),
            follow_ups: Vec::new(),
        }
    }

    /// Выборы события: 0 — много денег и год срока, 1 — немного денег, 2 — убыток и подорванное
    /// здоровье, 3 — деньги и смерть, 4 — эмиграция, 5 — уйти ни с чем, 6 — уйти ни с чем, но с
    /// продолжением наперёд.
    fn choices() -> Vec<Choice> {
        let mut choices: Vec<Choice> = vec![
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
            Consequences {
                end_life: Some(EndingId::new("emigration")),
                ..Consequences::default()
            },
            Consequences::default(),
            Consequences::default(),
        ]
        .into_iter()
        .map(choice)
        .collect();
        choices[6].follow_ups.push(FollowUp {
            event: EventId::new("raikom_offer"),
            after_days: 30,
            chance: 0.5,
        });
        choices
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
    fn an_unknown_continuation_weighs_against_a_choice_of_equal_gain() {
        let mut world = WorldRng::new(1);
        let dice = world.stream("bot");
        for style in [
            ChoiceStyle::Safest,
            ChoiceStyle::Greediest,
            ChoiceStyle::Sensible,
        ] {
            assert_eq!(pick(style, &[6, 5], dice), 5, "{style:?}");
            assert_eq!(pick(style, &[5, 6], dice), 5, "{style:?}");
        }
    }

    #[test]
    fn leaving_the_country_is_less_dangerous_than_death() {
        let mut world = WorldRng::new(1);
        let dice = world.stream("bot");
        assert_eq!(pick(ChoiceStyle::Safest, &[3, 4], dice), 4);
        assert_eq!(pick(ChoiceStyle::Sensible, &[4, 3], dice), 4);
        assert_eq!(
            pick(ChoiceStyle::Safest, &[0, 4], dice),
            0,
            "срок легче, чем конец жизни, даже по своей воле"
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
