//! Снимок мира для интерфейса: состояние жизни строками, готовыми к показу (§13.3). Подсистемы,
//! которых ещё нет в мире (события, связи, дела, зона), отдают пустые окна.

use super::Game;
use crate::economy::{Currency, MoneyRules};
use crate::time::GameDate;
use crate::view::{
    HeaderView, MarketRowView, MarketView, MoneyPlaceView, MoneyView, PocketItemView, PocketView,
    StateView, TimeView,
};

const CASH_LABEL: &str = "Наличные";
const CHECKS_HEADER_LABEL: &str = "Чеки";
const DOLLAR_RATE_HEADER_LABEL: &str = "Курс";
const CHECKS_POCKET_LABEL: &str = "Чеки ВПТ";
const DOLLARS_POCKET_LABEL: &str = "Доллары";
const CHECKS_RATE_LABEL: &str = "Чек ВПТ";
const DOLLAR_RATE_LABEL: &str = "Доллар";

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
        contacts: Vec::new(),
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

/// Карман: товар при себе, затем чеки и доллары.
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
    PocketView {
        items,
        danger: None,
    }
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
