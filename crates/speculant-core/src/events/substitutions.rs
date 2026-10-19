use super::Situation;
use crate::content::{Event, NpcId};
use crate::economy::{Currency, Markets};

const NPC: &str = "{npc}";
const PRICE: &str = "{price}";
const SUM: &str = "{sum}";

/// Подстановки в тексте события (§10.1): имя участника — человека из условия `npc` или соперника,
/// которого выбрал сид, — цена товара на толкучке города и сумма в номинале текущей валюты. Что
/// событие подставляет, проверяет загрузка контента.
pub struct Substitutions {
    npc: Option<String>,
    price: Option<String>,
    sum: Option<String>,
}

impl Substitutions {
    pub fn for_event(
        event: &Event,
        rival: Option<&NpcId>,
        situation: &Situation<'_>,
        markets: &Markets,
    ) -> Self {
        let content = situation.content;
        let rules = &content.money;
        Self {
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
        for (placeholder, value) in [(NPC, &self.npc), (PRICE, &self.price), (SUM, &self.sum)] {
            if let Some(value) = value {
                result = result.replace(placeholder, value);
            }
        }
        result
    }
}
