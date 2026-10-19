use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use super::{InspectionResponse, PolicyKind};
use crate::content::{BusinessKind, Content, PolicyOption};

/// Сто процентов зарплаты по рынку — у дела без политики зарплаты.
const MARKET_WAGE_PERCENT: u32 = 100;

/// Выбранные варианты политик дела — по одной на каждую политику его типа.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Policies {
    selected: BTreeMap<PolicyKind, usize>,
}

impl Policies {
    /// Варианты нового дела: у каждой политики его типа — вариант `default` из данных.
    pub fn defaults(kind: &BusinessKind, content: &Content) -> Self {
        Self {
            selected: kind
                .policies
                .iter()
                .map(|policy| {
                    let data = &content.policies[policy];
                    let index = data.option_index(&data.default).expect(
                        "вариант по умолчанию есть среди вариантов — это проверяет загрузка",
                    );
                    (*policy, index)
                })
                .collect(),
        }
    }

    /// Номер выбранного варианта; `None`, если у дела такой политики нет.
    pub fn selected(&self, policy: PolicyKind) -> Option<usize> {
        self.selected.get(&policy).copied()
    }

    /// Выбирает вариант; есть ли такая политика у дела и такой вариант у политики, проверяет
    /// вызывающий.
    pub fn set(&mut self, policy: PolicyKind, option: usize) {
        let selected = self
            .selected
            .get_mut(&policy)
            .expect("политика есть у дела — это проверяет вызывающий");
        *selected = option;
    }

    /// Выбранные варианты по порядку политик.
    pub fn chosen<'a>(
        &'a self,
        content: &'a Content,
    ) -> impl Iterator<Item = (PolicyKind, &'a PolicyOption)> + 'a {
        self.selected
            .iter()
            .map(|(policy, index)| (*policy, &content.policies[policy].options[*index]))
    }

    /// Сколько процентов выручки идёт крыше; ноль — крыши нет.
    pub fn roof_percent(&self, content: &Content) -> u32 {
        self.option(PolicyKind::Roof, content)
            .and_then(|option| option.roof_percent)
            .unwrap_or_default()
    }

    /// Что управляющий делает при проверке; `None` — у дела нет такой политики, и проверка
    /// всегда выходит за политики.
    pub fn inspection(&self, content: &Content) -> Option<InspectionResponse> {
        self.option(PolicyKind::Inspection, content)
            .and_then(|option| option.inspection)
    }

    /// Зарплата в процентах от рынка.
    pub fn wage_percent(&self, content: &Content) -> u32 {
        self.option(PolicyKind::Wage, content)
            .and_then(|option| option.wage_percent)
            .unwrap_or(MARKET_WAGE_PERCENT)
    }

    fn option<'c>(&self, policy: PolicyKind, content: &'c Content) -> Option<&'c PolicyOption> {
        self.selected
            .get(&policy)
            .map(|index| &content.policies[&policy].options[*index])
    }
}
