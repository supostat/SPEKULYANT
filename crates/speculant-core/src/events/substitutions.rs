use super::Situation;
use crate::business::Business;
use crate::content::{Event, NpcId};
use crate::economy::{Currency, Markets};

const NPC: &str = "{npc}";
const PRICE: &str = "{price}";
const SUM: &str = "{sum}";
const BUSINESS: &str = "{business}";
const MANAGER: &str = "{manager}";

/// Подстановки в тексте события (§10.1): имя участника — человека из условия `npc` или соперника,
/// которого выбрал сид, — цена товара на толкучке города, сумма в номинале текущей валюты, дело
/// происшествия и его управляющий. Что событие подставляет, проверяет загрузка контента.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Substitutions {
    npc: Option<String>,
    price: Option<String>,
    sum: Option<String>,
    business: Option<String>,
    manager: Option<String>,
}

impl Substitutions {
    pub fn for_event(
        event: &Event,
        rival: Option<&NpcId>,
        business: Option<&Business>,
        situation: &Situation<'_>,
        markets: &Markets,
    ) -> Self {
        let content = situation.content;
        let rules = &content.money;
        Self {
            business: business.map(|business| business.name(content)),
            manager: business
                .and_then(Business::manager)
                .map(|manager| content.npcs[manager.employee().npc()].name.clone()),
            npc: rival
                .or(event.conditions.npc.as_ref())
                .map(|npc| content.npcs[npc].name.clone()),
            price: event.price_of.as_ref().and_then(|good| {
                markets
                    .quote(content, situation.city, good, situation.today)
                    .map(|quote| rules.format_money(quote.buy, situation.today))
            }),
            sum: event.sum.map(|sum| {
                let amount = rules.from_real(sum, Currency::Ruble, situation.today);
                rules.format_money(amount, situation.today)
            }),
        }
    }

    /// Текст с подставленными значениями, без пустых строк по краям.
    pub fn apply(&self, text: &str) -> String {
        let mut result = text.trim().to_owned();
        for (placeholder, value) in [
            (NPC, &self.npc),
            (PRICE, &self.price),
            (SUM, &self.sum),
            (BUSINESS, &self.business),
            (MANAGER, &self.manager),
        ] {
            if let Some(value) = value {
                result = result.replace(placeholder, value);
            }
        }
        result
    }
}
