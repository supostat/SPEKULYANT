//! Зона в снимке (§12, §13.3 п. 9): срок, авторитет, сигареты и чай, услуги, передача, УДО, взятка
//! и доверенные лица на воле.

use super::super::Game;
use super::super::russian_count::{PACKS, count, packs, term};
use crate::economy::Currency;
use crate::time::GameDate;
use crate::view::{LevelView, PocketItemView, ZoneOfferView, ZoneServiceView, ZoneView};
use crate::zone::Zone;

const CIGARETTES: &str = "Сигареты";
const TEA: &str = "Чай";
const FROM: &str = "с";
const NOBODY: &str = "никого";

pub(super) fn zone(game: &Game, zone: &Zone, today: GameDate) -> ZoneView {
    let content = &game.content;
    let rules = &content.zone;
    let camp = content.camp_on(today);
    let stash = zone.stash();
    let cash = game.world.player.money().cash();
    let parcel_price = content
        .money
        .from_real(rules.parcel.price, Currency::Ruble, today);
    let parcel_due = today >= zone.next_parcel();
    let parole_from = zone.parole_from(camp);
    ZoneView {
        camp: camp.title.clone(),
        sentence: term(zone.sentence().months()),
        release: zone.sentence().until().numeric(),
        authority: LevelView {
            label: rules.authority_title(zone.authority()).to_owned(),
            level: zone.authority(),
            scale: rules.top_authority(),
        },
        stash: vec![
            PocketItemView {
                label: CIGARETTES.to_owned(),
                amount: count(stash.cigarettes, PACKS),
            },
            PocketItemView {
                label: TEA.to_owned(),
                amount: count(stash.tea, PACKS),
            },
        ],
        working: zone.is_working(),
        services: content
            .zone_services
            .iter()
            .map(|(id, service)| ZoneServiceView {
                service: id.to_string(),
                title: service.title.clone(),
                price: packs(service.price),
                available: stash.covers(service.price),
            })
            .collect(),
        parcel: ZoneOfferView {
            terms: if parcel_due {
                content.money.format_money(parcel_price, today)
            } else {
                format!("{FROM} {}", zone.next_parcel().numeric())
            },
            available: parcel_due && cash.covers(parcel_price),
        },
        parole: ZoneOfferView {
            terms: format!("{FROM} {}", parole_from.numeric()),
            available: today >= parole_from,
        },
        bribe: camp.bribe.map(|bribe| {
            let price = content.money.from_real(bribe, Currency::Ruble, today);
            ZoneOfferView {
                terms: content.money.format_money(price, today),
                available: cash.covers(price),
            }
        }),
        trustees: game
            .world
            .businesses
            .values()
            .map(|business| {
                let manager = business.manager().map_or(NOBODY, |manager| {
                    content.npcs[manager.employee().npc()].name.as_str()
                });
                format!("{} — {manager}", business.name(content))
            })
            .collect(),
    }
}
