use std::collections::BTreeMap;

use serde::Deserialize;

use super::{BusinessForm, BusinessFormId, GoodId};
use crate::business::PolicyKind;
use crate::economy::Real;
use crate::time::GameDate;

/// Тип дела (§7.1) из `data/business/kinds/<тип>.toml`: ларьки, магазин, цех, челночный рейс,
/// банк. Деньги — рубли 1984 года за точку в месяц.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BusinessKind {
    pub title: String,
    /// Формы собственности, в которых такое дело бывает; на каждую дату дело носит открытую форму
    /// с самым поздним началом.
    pub forms: Vec<BusinessFormId>,
    /// Цена открытия точки.
    pub point_cost: Real,
    pub max_points: u32,
    /// Аренда точки; у челночного рейса её нет.
    #[serde(default)]
    pub rent: Real,
    /// Зарплата персонала точки по рынку.
    pub wages: Real,
    /// Что точка продаёт за месяц: товар и штуки по средней цене города.
    #[serde(default)]
    pub goods: BTreeMap<GoodId, u32>,
    /// Доля продаж, которая остаётся выручкой.
    #[serde(default)]
    pub margin: f64,
    /// Выручка точки без товара — оборот банка.
    #[serde(default)]
    pub income: Real,
    /// Политики, которые управляющий этого дела получает.
    pub policies: Vec<PolicyKind>,
    /// Сдвиги уровней наезда и воровства у такого дела.
    #[serde(default)]
    pub raid: i32,
    #[serde(default)]
    pub theft: i32,
}

impl BusinessKind {
    /// Форма, которую дело носит в этот день; `None`, пока ни одна из его форм не открыта.
    pub fn form_on<'a>(
        &self,
        forms: &'a BTreeMap<BusinessFormId, BusinessForm>,
        date: GameDate,
    ) -> Option<(&'a BusinessFormId, &'a BusinessForm)> {
        self.forms
            .iter()
            .filter_map(|id| forms.get_key_value(id))
            .filter(|(_, form)| form.is_open_on(date))
            .max_by_key(|(_, form)| form.from)
    }

    pub fn has_policy(&self, policy: PolicyKind) -> bool {
        self.policies.contains(&policy)
    }
}
