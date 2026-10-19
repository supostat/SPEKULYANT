//! Последствия выбора (§10.1): деньги, товар, репутация, связи, флаги, сдвиг цен и перемены в деле
//! — и строка того, что изменилось, для окна исхода.

use super::Game;
use crate::business::BusinessId;
use crate::content::{BusinessChange, Consequences, ContactChange, Content, NpcId};
use crate::economy::{Currency, Real};
use crate::reputation::Contact;
use crate::time::GameDate;

const MONEY: &str = "Деньги";
const HEALTH: &str = "Здоровье";
const NOTHING_CHANGED: &str = "Ничего не изменилось.";
const CONTACT_OWES: &str = "за ним услуг";
const PLAYER_OWES: &str = "за тобой услуг";
const AWAY_UNTIL: &str = "не на связи до";
const LOST: &str = "потерян";
const MANAGER_LEAVES: &str = "уходит";
const CLOSED_UNTIL: &str = "закрыто до";
const BUSINESS_LOST: &str = "дело потеряно";
const PLUS: char = '+';
const MINUS: char = '−';

impl Game {
    /// Применяет последствия и возвращает строку перемен: «Торговцы +1. Деньги −600 р.». Флаги и
    /// сдвиги цен игроку не показываются — о них говорит текст исхода; здоровье — словом
    /// состояния. Финал последствия или кончившееся здоровье завершают жизнь после всех перемен. `rival` — участвующий в
    /// событии соперник, к которому относится последствие `rival`; `business` — дело
    /// происшествия, к которому относится последствие `business`.
    pub(super) fn apply_consequences(
        &mut self,
        consequences: &Consequences,
        what: &str,
        rival: Option<&NpcId>,
        business: Option<&BusinessId>,
        today: GameDate,
    ) -> String {
        let content = std::sync::Arc::clone(&self.content);
        let mut effects = Vec::new();
        for change in &consequences.reputation {
            self.reputation
                .change(&change.faction, change.change, content.reputation);
            let title = &content.factions[&change.faction].title;
            effects.push(format!("{title} {}", signed(change.change)));
        }
        if let Some(money) = consequences.money {
            let amount = Real::from_kopecks(money.kopecks().abs());
            let amount = content.money.from_real(amount, Currency::Ruble, today);
            let sign = if money.is_negative() {
                self.player.pay_cash(amount);
                MINUS
            } else {
                self.player.receive_cash(amount);
                PLUS
            };
            effects.push(format!(
                "{MONEY} {sign}{}",
                content.money.format_money(amount, today)
            ));
        }
        for change in &consequences.goods {
            let quantity = change.change.unsigned_abs();
            if change.change < 0 {
                let held = self.player.pocket().quantity(&change.good);
                self.chronicle.gave_away(&change.good, quantity, held);
                self.player.remove_goods(&change.good, quantity);
            } else {
                self.player.add_goods(&change.good, quantity);
            }
            let name = &content.goods[&change.good].name;
            effects.push(format!("{name} {}", signed(change.change)));
        }
        for change in &consequences.contacts {
            self.contacts
                .entry(change.npc.clone())
                .or_insert_with(|| Contact::new(change.npc.clone()))
                .apply(change, content.reputation, today, what);
            let name = &content.npcs[&change.npc].name;
            relation_effects(&mut effects, name, change.relation, change.favours);
            if let Some(days) = change.away_days {
                effects.push(format!(
                    "{name}: {AWAY_UNTIL} {}",
                    today.after_days(days).numeric()
                ));
            }
            if change.lost {
                effects.push(format!("{name}: {LOST}"));
            }
        }
        if let (Some(change), Some(npc)) = (consequences.rival, rival) {
            let as_contact = ContactChange {
                npc: npc.clone(),
                relation: change.relation,
                favours: change.favours,
                away_days: None,
                lost: false,
            };
            self.rivals
                .get_mut(npc)
                .expect("участвующий соперник есть в мире")
                .memory_mut()
                .apply(&as_contact, content.reputation, today, what);
            relation_effects(
                &mut effects,
                &content.npcs[npc].name,
                change.relation,
                change.favours,
            );
        }
        if let (Some(change), Some(id)) = (&consequences.business, business) {
            self.change_business(&content, change, id, today, &mut effects);
        }
        if consequences.health != 0 {
            let rules = &content.life;
            self.character
                .change_health(consequences.health, rules.health_max);
            effects.push(format!(
                "{HEALTH}: {}",
                rules.health_state(self.character.health())
            ));
        }
        for flag in &consequences.set_flags {
            self.events.set_flag(flag.clone());
        }
        for flag in &consequences.clear_flags {
            self.events.clear_flag(flag);
        }
        for shock in &consequences.price_shocks {
            self.markets.apply_shock(
                shock.good.clone(),
                shock.city.clone(),
                shock.factor,
                shock.days,
                today,
            );
        }
        let ending = match &consequences.end_life {
            Some(ending) => Some(ending),
            None if self.character.health() <= 0 => Some(&content.life.health_ending),
            None => None,
        };
        if let Some(ending) = ending {
            self.end_life(ending, today);
        }
        if effects.is_empty() {
            NOTHING_CHANGED.to_owned()
        } else {
            sentences(&effects)
        }
    }
}

impl Game {
    /// Перемена в деле происшествия: варианты политик, которые у этого дела есть, уход
    /// управляющего, закрытие на срок, потеря дела.
    fn change_business(
        &mut self,
        content: &Content,
        change: &BusinessChange,
        id: &BusinessId,
        today: GameDate,
        effects: &mut Vec<String>,
    ) {
        let business = self
            .businesses
            .get_mut(id)
            .expect("дело происшествия живо, пока событие ждёт ответа");
        let name = business.name(content);
        for setting in &change.policies {
            if !business.kind(content).has_policy(setting.policy) {
                continue;
            }
            let policy = &content.policies[&setting.policy];
            let option = policy
                .option_index(&setting.option)
                .expect("вариант политики есть в данных — это проверяет загрузка");
            business.set_policy(setting.policy, option);
            effects.push(format!("{name}: {} — {}", policy.title, setting.option));
        }
        if change.manager_leaves
            && let Some(manager) = business.dismiss()
        {
            let person = &content.npcs[manager.employee().npc()].name;
            effects.push(format!("{name}: {person} {MANAGER_LEAVES}"));
        }
        if let Some(days) = change.closed_days {
            business.close_for(days, today);
            let until = business
                .closed_until()
                .expect("дело только что закрыли до даты");
            effects.push(format!("{name}: {CLOSED_UNTIL} {}", until.numeric()));
        }
        if change.lost {
            self.businesses.remove(id);
            effects.push(format!("{name}: {BUSINESS_LOST}"));
        }
    }
}

/// Перемены предложениями: «Торговцы +1. Деньги −600 р. Водка, ящик +1.» — точка сокращения
/// «р.» заканчивает предложение сама.
fn sentences(effects: &[String]) -> String {
    let mut text = String::new();
    for effect in effects {
        if !text.is_empty() {
            text.push(' ');
        }
        text.push_str(effect);
        if !effect.ends_with('.') {
            text.push('.');
        }
    }
    text
}

/// Перемены в отношениях с человеком: «Кофи −2», «Кофи: за ним услуг +1».
fn relation_effects(effects: &mut Vec<String>, name: &str, relation: i32, favours: i32) {
    if relation != 0 {
        effects.push(format!("{name} {}", signed(relation)));
    }
    match favours.signum() {
        1 => effects.push(format!("{name}: {CONTACT_OWES} {}", signed(favours))),
        -1 => effects.push(format!(
            "{name}: {PLAYER_OWES} {PLUS}{}",
            favours.unsigned_abs()
        )),
        _ => {}
    }
}

/// Число со знаком так, как его пишет окно исхода: «+1», «−3».
pub(super) fn signed(value: i32) -> String {
    let sign = if value < 0 { MINUS } else { PLUS };
    format!("{sign}{}", value.unsigned_abs())
}
