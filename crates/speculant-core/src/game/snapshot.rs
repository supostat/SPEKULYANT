//! Снимок мира для интерфейса: состояние жизни строками, готовыми к показу (§13.3). Подсистемы,
//! которых ещё нет в мире (события, дела, зона), отдают пустые окна.

use std::cmp::Ordering;

use super::Game;
use crate::economy::{Currency, MoneyRules};
use crate::reputation::{Availability, Contact};
use crate::time::GameDate;
use crate::view::{
    ContactView, FactionView, HeaderView, LevelView, MarketRowView, MarketView, MoneyPlaceView,
    MoneyView, PocketItemView, PocketView, StateView, TimeView,
};

const CASH_LABEL: &str = "Наличные";
const CHECKS_HEADER_LABEL: &str = "Чеки";
const DOLLAR_RATE_HEADER_LABEL: &str = "Курс";
const CHECKS_POCKET_LABEL: &str = "Чеки ВПТ";
const DOLLARS_POCKET_LABEL: &str = "Доллары";
const CHECKS_RATE_LABEL: &str = "Чек ВПТ";
const DOLLAR_RATE_LABEL: &str = "Доллар";
const CONTACTS_POCKET_LABEL: &str = "Связей";
const WELL_DISPOSED: &str = "расположен";
const ACQUAINTED: &str = "знакомый";
const OFFENDED: &str = "обижен";
const PLAYER_OWES: &str = "За тобой услуг";
const CONTACT_OWES: &str = "За ним услуг";
const REACHABLE: &str = "на связи";
const AWAY_UNTIL: &str = "не на связи до";
const LOST: &str = "потерян";

pub(super) fn snapshot(game: &Game) -> StateView {
    let today = game.clock.today();
    StateView {
        screen: game.screen,
        header: header(game, today),
        time: TimeView {
            speed: game.clock.speed(),
            paused: game.clock.is_paused(),
        },
        market: market(game, today),
        pocket: pocket(game, today),
        rumors: Vec::new(),
        messages: game.messages.clone(),
        pending_event: None,
        pager: None,
        businesses: Vec::new(),
        manager: None,
        contacts: game
            .contacts
            .values()
            .map(|contact| contact_view(game, contact, today))
            .collect(),
        factions: factions(game, today),
        money: money(game, today),
        notebook: Vec::new(),
        zone: None,
        summary: None,
        biographies: Vec::new(),
    }
}

/// Шапка: город, дата, наличные; чеки, доллары и курс — когда они есть у игрока.
fn header(game: &Game, today: GameDate) -> HeaderView {
    let rules = &game.content.money;
    let wallet = game.player.money();
    let mut money = vec![format!(
        "{CASH_LABEL}: {}",
        rules.format_money(wallet.cash(), today)
    )];
    let checks = wallet.holdings(Currency::Checks);
    if !checks.is_zero() {
        money.push(format!(
            "{CHECKS_HEADER_LABEL}: {}",
            rules.format_number(checks, today)
        ));
    }
    let dollars = wallet.holdings(Currency::Dollar);
    if !dollars.is_zero() {
        money.push(rules.format_money(dollars, today));
        money.push(format!(
            "{DOLLAR_RATE_HEADER_LABEL}: {}",
            rate(rules, Currency::Dollar, today)
        ));
    }
    HeaderView {
        city: game.content.cities[&game.city].name.clone(),
        date: today.russian(),
        money,
    }
}

/// Толкучка города: товары, которые уже появились, с ценниками следующей штуки и запасом.
fn market(game: &Game, today: GameDate) -> MarketView {
    let content = &game.content;
    let goods = content
        .goods
        .iter()
        .filter_map(|(id, good)| {
            let quote = game.markets.quote(content, &game.city, id, today)?;
            Some(MarketRowView {
                good: id.to_string(),
                name: good.name.clone(),
                buy: content.money.format_number(quote.buy, today),
                sell: content.money.format_number(quote.sell, today),
                stock: quote.stock.to_string(),
            })
        })
        .collect();
    MarketView {
        place: content.cities[&game.city].market.clone(),
        goods,
    }
}

/// Карман: товар при себе, чеки и доллары, число связей и угроза эпохи.
fn pocket(game: &Game, today: GameDate) -> PocketView {
    let content = &game.content;
    let mut items: Vec<PocketItemView> = game
        .player
        .pocket()
        .goods()
        .map(|(id, quantity)| {
            let good = &content.goods[id];
            PocketItemView {
                label: good.short_name.clone(),
                amount: format!("{quantity} {}", good.unit),
            }
        })
        .collect();
    let wallet = game.player.money();
    let checks = wallet.holdings(Currency::Checks);
    if !checks.is_zero() {
        items.push(PocketItemView {
            label: CHECKS_POCKET_LABEL.to_owned(),
            amount: content.money.format_number(checks, today),
        });
    }
    let dollars = wallet.holdings(Currency::Dollar);
    if !dollars.is_zero() {
        items.push(PocketItemView {
            label: DOLLARS_POCKET_LABEL.to_owned(),
            amount: content.money.format_money(dollars, today),
        });
    }
    let reachable = game
        .contacts
        .values()
        .filter(|contact| !contact.is_lost())
        .count();
    items.push(PocketItemView {
        label: CONTACTS_POCKET_LABEL.to_owned(),
        amount: reachable.to_string(),
    });
    let threat = &content.epoch_profile_on(today).threat;
    PocketView {
        items,
        danger: Some(LevelView {
            label: threat.label.clone(),
            level: game
                .reputation
                .hostility(&threat.faction, content.reputation),
            scale: content.reputation.limit,
        }),
    }
}

/// Связь: кто это, как к игроку относится, кто кому должен и можно ли до него достучаться.
fn contact_view(game: &Game, contact: &Contact, today: GameDate) -> ContactView {
    let template = &game.content.npcs[contact.npc()];
    let relation = match contact.relation().cmp(&0) {
        Ordering::Greater => WELL_DISPOSED,
        Ordering::Equal => ACQUAINTED,
        Ordering::Less => OFFENDED,
    };
    let debt = match contact.debt().cmp(&0) {
        Ordering::Less => Some(format!("{PLAYER_OWES}: {}", contact.debt().unsigned_abs())),
        Ordering::Equal => None,
        Ordering::Greater => Some(format!("{CONTACT_OWES}: {}", contact.debt())),
    };
    let availability = match contact.availability() {
        Availability::AwayUntil(until) if today < until => {
            format!("{AWAY_UNTIL} {}", until.numeric())
        }
        Availability::AwayUntil(_) | Availability::Available => REACHABLE.to_owned(),
        Availability::Lost => LOST.to_owned(),
    };
    ContactView {
        contact: contact.npc().to_string(),
        name: template.name.clone(),
        role: template.role.clone(),
        relation: relation.to_owned(),
        debt,
        availability,
        history: contact.history().to_vec(),
    }
}

/// Репутация у фракций, которые уже есть в мире: силовики появятся только со стабилизацией.
fn factions(game: &Game, today: GameDate) -> Vec<FactionView> {
    let content = &game.content;
    content
        .factions
        .iter()
        .filter(|(_, faction)| faction.is_visible_on(today, &content.time.epochs))
        .map(|(id, faction)| FactionView {
            faction: id.to_string(),
            title: faction.title.clone(),
            level: game.reputation.level(id),
            limit: content.reputation.limit,
        })
        .collect()
}

/// Деньги: места, куда сейчас можно положить или где что-то лежит, и курсы их валют.
fn money(game: &Game, today: GameDate) -> MoneyView {
    let content = &game.content;
    let wallet = game.player.money();
    let shown: Vec<_> = content
        .money_places
        .iter()
        .filter(|(id, place)| place.accepts_deposits_on(today) || !wallet.balance(id).is_zero())
        .collect();
    let places = shown
        .iter()
        .map(|(id, place)| MoneyPlaceView {
            place: id.to_string(),
            title: place.title.clone(),
            amount: content.money.format_money(wallet.balance(id), today),
            risk: place.risk.clone(),
        })
        .collect();
    let rates = [
        (Currency::Checks, CHECKS_RATE_LABEL),
        (Currency::Dollar, DOLLAR_RATE_LABEL),
    ]
    .into_iter()
    .filter(|(currency, _)| shown.iter().any(|(_, place)| place.currency == *currency))
    .map(|(currency, label)| format!("{label}: {}", rate(&content.money, currency, today)))
    .collect();
    MoneyView { places, rates }
}

/// Курс валюты в рублях дня: «3.50 р.», «4 455 р.».
fn rate(rules: &MoneyRules, currency: Currency, today: GameDate) -> String {
    let one = rules.whole_units(currency, 1, today);
    rules.format_money(rules.exchange(one, Currency::Ruble, today), today)
}
