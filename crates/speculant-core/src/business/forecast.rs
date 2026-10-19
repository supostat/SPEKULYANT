use super::{Employee, Policies};
use crate::content::{BusinessForm, BusinessKind, Content, Habit};

/// Прогноз на месяц с экрана управляющего (§7.3): насколько политики и привычки управляющего
/// сдвигают выручку, уровни риска наезда и воровства. Уровни — те же, по которым дело разыгрывает
/// происшествия, поэтому прогноз не врёт.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Forecast {
    /// Сдвиг выручки в процентах.
    pub revenue_percent: i32,
    pub raid: u8,
    pub theft: u8,
    /// Лояльность управляющего с поправкой политик — от неё зависят уход и предательство.
    pub loyalty: u8,
}

impl Forecast {
    /// Наезд: база правил, тип дела, форма собственности, политики и привычки. Воровство: база,
    /// честность управляющего против середины шкалы, тип дела, политики и привычки. Выручка —
    /// сумма сдвигов политик и привычек.
    pub fn new(
        content: &Content,
        kind: &BusinessKind,
        form: &BusinessForm,
        policies: &Policies,
        employee: &Employee,
    ) -> Self {
        let rules = &content.business;
        let habits: Vec<&Habit> = employee
            .habits()
            .iter()
            .map(|habit| &content.habits[habit])
            .collect();
        let options: Vec<_> = policies.chosen(content).map(|(_, option)| option).collect();
        let revenue_percent = options.iter().map(|option| option.revenue).sum::<i32>()
            + habits.iter().map(|habit| habit.revenue).sum::<i32>();
        let raid = rules.base_raid
            + kind.raid
            + form.raid
            + options.iter().map(|option| option.raid).sum::<i32>()
            + habits.iter().map(|habit| habit.raid).sum::<i32>();
        let dishonesty = i32::from(rules.neutral_trait) - i32::from(employee.honesty());
        let theft = rules.base_theft
            + dishonesty
            + kind.theft
            + options.iter().map(|option| option.theft).sum::<i32>()
            + habits.iter().map(|habit| habit.theft).sum::<i32>();
        let loyalty = i32::from(employee.loyalty())
            + options.iter().map(|option| option.loyalty).sum::<i32>();
        Self {
            revenue_percent,
            raid: rules.risk_level(raid),
            theft: rules.risk_level(theft),
            loyalty: rules.trait_level(loyalty),
        }
    }
}
