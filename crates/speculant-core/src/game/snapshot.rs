//! Снимок мира для интерфейса: состояние жизни строками, готовыми к показу (§13.3).

mod business;
mod life;
mod meta;
mod zone;

use std::cmp::Ordering;

use super::Game;
use crate::content::{Content, Pitch};
use crate::economy::{Currency, MoneyRules};
use crate::meta::Meta;
use crate::npc::Rival;
use crate::reputation::{Availability, Contact};
use crate::time::GameDate;
use crate::travel::Access;
use crate::view::{
    ChoiceView, ContactView, DestinationView, EventView, FactionView, HeaderView, JourneyView,
    LevelView, LobbyView, MarketRowView, MarketView, MoneyPlaceView, MoneyView, OutcomeView,
    PagerMessageView, PagerView, PocketItemView, PocketView, StateView, TimeView, ToneView,
    TravelView,
};

const CASH_LABEL: &str = "Наличные";
const CHECKS_HEADER_LABEL: &str = "Чеки";
const DOLLAR_RATE_HEADER_LABEL: &str = "Курс";
const CHECKS_POCKET_LABEL: &str = "Чеки ВПТ";
const DOLLARS_POCKET_LABEL: &str = "Доллары";
const CHECKS_RATE_LABEL: &str = "Чек ВПТ";
const DOLLAR_RATE_LABEL: &str = "Доллар";
const CONTACTS_POCKET_LABEL: &str = "Связей";
const ON_THE_ROAD: &str = "В дороге";
const IN_ZONE: &str = "В зоне";
const WELL_DISPOSED: &str = "расположен";
const ACQUAINTED: &str = "знакомый";
const OFFENDED: &str = "обижен";
const PLAYER_OWES: &str = "За тобой услуг";
const CONTACT_OWES: &str = "За ним услуг";
const REACHABLE: &str = "на связи";
const AWAY_UNTIL: &str = "не на связи до";
const LOST: &str = "потерян";
/// Номер абонента — четыре цифры из сида жизни.
const FIRST_PAGER_NUMBER: u64 = 1000;
const PAGER_NUMBERS: u64 = 9000;

pub(super) fn snapshot(game: &Game) -> StateView {
    let today = game.world.clock.today();
    StateView {
        screen: game.world.screen,
        ironman: game.world.ironman,
        header: header(game, today),
        character: life::character(game, today),
        homes: life::homes(game, today),
        exits: life::exits(game, today),
        time: TimeView {
            speed: game.world.clock.speed(),
            paused: game.world.clock.is_paused()
                || game.world.events.pending().is_some()
                || game.world.summary.is_some(),
            fast_forward: game.world.clock.fast_forward(),
        },
        market: market(game, today),
        travel: travel(game, today),
        pocket: pocket(game, today),
        rumors: meta::rumors(game, today),
        messages: game.world.messages.clone(),
        pending_event: pending_event(game, today),
        pager: pager(game),
        businesses: business::businesses(game, today),
        candidates: business::candidates(game, today),
        business_kinds: business::kinds(game, today),
        contacts: game
            .world
            .contacts
            .values()
            .map(|contact| contact_view(game, contact, today))
            .chain(
                game.world
                    .rivals
                    .iter()
                    .map(|rival| rival_view(game, rival)),
            )
            .collect(),
        factions: factions(game, today),
        money: money(game, today),
        notebook: meta::notebook(&game.content, &game.meta),
        records: meta::records(&game.content, &game.meta),
        zone: game
            .world
            .zone
            .as_ref()
            .map(|zone| zone::zone(game, zone, today)),
        summary: game
            .world
            .summary
            .as_ref()
            .map(|summary| life::summary(game, summary)),
        biographies: life::biographies(&game.content, &game.meta),
    }
}

/// Лобби: биографии, записная книжка и рекорды зависят только от данных и метапрогрессии, мелодия
/// заставки — от данных.
pub(super) fn lobby(content: &Content, meta: &Meta) -> LobbyView {
    LobbyView {
        biographies: life::biographies(content, meta),
        notebook: meta::notebook(content, meta),
        records: meta::records(content, meta),
        boot_melody: content
            .boot_melody
            .tones()
            .into_iter()
            .map(|(pitch, milliseconds)| ToneView {
                hertz: match pitch {
                    Pitch::Rest => None,
                    Pitch::Hertz(hertz) => Some(hertz),
                },
                milliseconds,
            })
            .collect(),
    }
}

/// Событие, остановившее время: текст и выборы с подстановками, а после выбора — исход.
fn pending_event(game: &Game, today: GameDate) -> Option<EventView> {
    let pending = game.world.events.pending()?;
    let event = &game.content.events[&pending.event];
    let situation = game.situation(today);
    let substitutions = &pending.substitutions;
    Some(EventView {
        title: event.title.clone(),
        text: substitutions.apply(&event.text),
        choices: event
            .choices
            .iter()
            .map(|choice| ChoiceView {
                text: substitutions.apply(&choice.text),
                available: situation.allows_choice(choice),
            })
            .collect(),
        outcome: pending.resolution.as_ref().map(|resolution| {
            let choice = &event.choices[resolution.choice];
            OutcomeView {
                text: substitutions.apply(choice.outcome.as_deref().unwrap_or_default()),
                effects: resolution.effects.clone(),
            }
        }),
    })
}

/// Шапка: город, дата, наличные; чеки, доллары и курс — когда они есть у игрока.
fn header(game: &Game, today: GameDate) -> HeaderView {
    let rules = &game.content.money;
    let wallet = game.world.player.money();
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
    let city = match (&game.world.journey, &game.world.zone) {
        (_, Some(_)) => IN_ZONE.to_owned(),
        (Some(_), None) => ON_THE_ROAD.to_owned(),
        (None, None) => game.content.cities[&game.world.city].name.clone(),
    };
    HeaderView {
        city,
        date: today.russian(),
        money,
    }
}

/// Толкучка города: товары, которые уже появились, с ценниками следующей штуки и запасом. В
/// дороге и в зоне толкучки нет.
fn market(game: &Game, today: GameDate) -> MarketView {
    let content = &game.content;
    if game.world.zone.is_some() {
        return MarketView {
            place: content.camp_on(today).title.clone(),
            goods: Vec::new(),
        };
    }
    if let Some(journey) = &game.world.journey {
        return MarketView {
            place: format!(
                "{ON_THE_ROAD}: {} — {}",
                content.cities[&journey.from].name, content.cities[&journey.to].name
            ),
            goods: Vec::new(),
        };
    }
    let goods = content
        .goods
        .iter()
        .filter_map(|(id, good)| {
            let quote = game
                .world
                .markets
                .quote(content, &game.world.city, id, today)?;
            Some(MarketRowView {
                good: id.to_string(),
                name: good.name.clone(),
                buy: content.money.format_number(quote.buy, today),
                sell: content.money.format_number(quote.sell, today),
                stock: quote.stock.to_string(),
                held: match game.world.player.pocket().quantity(id) {
                    0 => String::new(),
                    quantity => quantity.to_string(),
                },
            })
        })
        .collect();
    MarketView {
        place: content.cities[&game.world.city].market.clone(),
        goods,
    }
}

/// Поездка: в пути — откуда, куда и когда приезд; дома — открытые направления из города с днями
/// и ценой в номинале дня. Закрытые города не показываются, из зоны не ездят.
fn travel(game: &Game, today: GameDate) -> TravelView {
    let content = &game.content;
    if game.world.zone.is_some() {
        return TravelView {
            journey: None,
            destinations: Vec::new(),
        };
    }
    if let Some(journey) = &game.world.journey {
        let days_left = journey.arrives.day_index() - today.day_index();
        return TravelView {
            journey: Some(JourneyView {
                from: content.cities[&journey.from].name.clone(),
                to: content.cities[&journey.to].name.clone(),
                arrives: journey.arrives.russian(),
                days_left: u32::try_from(days_left)
                    .expect("в пути приезд не раньше сегодняшнего дня"),
            }),
            destinations: Vec::new(),
        };
    }
    let cash = game.world.player.money().cash();
    let someone_helps = game
        .world
        .contacts
        .values()
        .any(|contact| contact.refusal(content.reputation, today).is_none());
    let destinations = content
        .routes
        .iter()
        .filter(|route| route.from == game.world.city)
        .filter_map(|route| {
            let access = Access::of(route, content, today);
            if access == Access::Closed {
                return None;
            }
            let fare = content.money.from_real(route.fare, Currency::Ruble, today);
            let by_connections = access == Access::ByConnections;
            Some(DestinationView {
                city: route.to.to_string(),
                name: content.cities[&route.to].name.clone(),
                days: route.days,
                fare: content.money.format_money(fare, today),
                by_connections,
                available: cash.covers(fare) && (!by_connections || someone_helps),
            })
        })
        .collect();
    TravelView {
        journey: None,
        destinations,
    }
}

/// Карман: товар при себе, чеки и доллары, число связей и угроза эпохи.
fn pocket(game: &Game, today: GameDate) -> PocketView {
    let content = &game.content;
    let mut items: Vec<PocketItemView> = game
        .world
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
    let wallet = game.world.player.money();
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
        .world
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
                .world
                .reputation
                .hostility(&threat.faction, content.reputation),
            scale: content.reputation.limit,
        }),
    }
}

/// Связь: кто это, как к игроку относится, кто кому должен и можно ли до него достучаться.
fn contact_view(game: &Game, contact: &Contact, today: GameDate) -> ContactView {
    let availability = match contact.availability() {
        Availability::AwayUntil(until) if today < until => {
            format!("{AWAY_UNTIL} {}", until.numeric())
        }
        Availability::AwayUntil(_) | Availability::Available => REACHABLE.to_owned(),
        Availability::Lost => LOST.to_owned(),
    };
    person_view(game, contact, false, availability)
}

/// Соперник: то же, что связь, только вместо доступности — судьба.
fn rival_view(game: &Game, rival: &Rival) -> ContactView {
    person_view(game, rival.memory(), true, rival.fate().status())
}

fn person_view(game: &Game, memory: &Contact, rival: bool, availability: String) -> ContactView {
    let template = &game.content.npcs[memory.npc()];
    let relation = match memory.relation().cmp(&0) {
        Ordering::Greater => WELL_DISPOSED,
        Ordering::Equal => ACQUAINTED,
        Ordering::Less => OFFENDED,
    };
    let debt = match memory.debt().cmp(&0) {
        Ordering::Less => Some(format!("{PLAYER_OWES}: {}", memory.debt().unsigned_abs())),
        Ordering::Equal => None,
        Ordering::Greater => Some(format!("{CONTACT_OWES}: {}", memory.debt())),
    };
    ContactView {
        contact: memory.npc().to_string(),
        name: template.name.clone(),
        role: template.role.clone(),
        rival,
        relation: relation.to_owned(),
        debt,
        availability,
        history: memory.history().to_vec(),
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
            level: game.world.reputation.level(id),
            limit: content.reputation.limit,
        })
        .collect()
}

/// Деньги: места, куда сейчас можно положить или где что-то лежит, и курсы их валют.
fn money(game: &Game, today: GameDate) -> MoneyView {
    let content = &game.content;
    let wallet = game.world.player.money();
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

/// Пейджер — если устройство при игроке: заголовок с номером абонента и лента от старых сообщений к
/// новым, заглавными, как их показывает экран пейджера.
fn pager(game: &Game) -> Option<PagerView> {
    let device = game.pager_device()?;
    let number = FIRST_PAGER_NUMBER + game.world.rng.seed() % PAGER_NUMBERS;
    Some(PagerView {
        device: device.title.replace("{number}", &number.to_string()),
        messages: game
            .world
            .pager
            .messages()
            .iter()
            .map(|message| PagerMessageView {
                time: message.date.day_and_month(),
                text: message.text.to_uppercase(),
                urgent: message.urgent,
            })
            .collect(),
    })
}
