//! Жизнь в снимке: персонаж, жильё, добровольные уходы, биографии на экране выбора и итоги.

use super::super::Game;
use super::super::russian_count::{MONTHS_IN_YEAR, YEARS, count, term};
use crate::content::Content;
use crate::economy::Currency;
use crate::life::{DealRecord, LifeSummary};
use crate::meta::Meta;
use crate::time::GameDate;
use crate::view::{
    BiographyView, CharacterView, ExitView, HomeView, LifeSummaryView, SummaryLineView,
};

const LESS_THAN_A_MONTH: &str = "меньше месяца";
const NOTHING: &str = "—";
const BEST_DEAL: &str = "Лучшая сделка";
const WORST_DEAL: &str = "Худшая сделка";
const LIVED: &str = "Прожито";
const BUSINESSES: &str = "Дела";
const BUSINESS_SEPARATOR: &str = "; ";
const OPENED_BIOGRAPHY: &str = "Открыто: биография";

pub(super) fn character(game: &Game, today: GameDate) -> CharacterView {
    let content = &game.content;
    let character = &game.world.character;
    CharacterView {
        name: character.name().to_owned(),
        age: count(character.age_on(today), YEARS),
        health: content.life.health_state(character.health()).to_owned(),
        housing: content.housing[character.housing()].title.clone(),
    }
}

/// Нынешнее жильё и то, что продаётся сегодня, — от дешёвого к дорогому.
pub(super) fn homes(game: &Game, today: GameDate) -> Vec<HomeView> {
    let content = &game.content;
    let current = game.world.character.housing();
    let resale = game.resale_of_home(today);
    let cash = game.world.player.money().cash();
    let mut homes: Vec<_> = content
        .housing
        .iter()
        .filter(|(id, housing)| *id == current || housing.is_on_sale(today))
        .collect();
    homes.sort_by_key(|(_, housing)| housing.price);
    homes
        .into_iter()
        .map(|(id, housing)| {
            let is_current = id == current;
            let price = housing
                .price
                .map(|price| content.money.from_real(price, Currency::Ruble, today));
            let affordable = price.is_some_and(|price| cash.plus(resale).covers(price));
            HomeView {
                housing: id.to_string(),
                title: housing.title.clone(),
                price: price.map(|price| content.money.format_money(price, today)),
                current: is_current,
                available: !is_current
                    && housing.is_on_sale(today)
                    && affordable
                    && game.world.journey.is_none()
                    && game.world.zone.is_none(),
            }
        })
        .collect()
}

/// Добровольные уходы; доступны, когда открыты и игрок в городе, а не в дороге и не в зоне.
pub(super) fn exits(game: &Game, today: GameDate) -> Vec<ExitView> {
    game.content
        .endings
        .iter()
        .filter(|(_, ending)| ending.voluntary)
        .map(|(id, ending)| ExitView {
            ending: id.to_string(),
            title: ending.title.clone(),
            available: ending.is_open_on(today)
                && game.world.journey.is_none()
                && game.world.zone.is_none(),
        })
        .collect()
}

/// Все биографии; закрытые — с подсказкой, чем их открыть.
pub(super) fn biographies(content: &Content, meta: &Meta) -> Vec<BiographyView> {
    content
        .biographies
        .iter()
        .map(|(id, biography)| {
            let unlocked = biography.starts_unlocked || meta.unlocked().contains(id);
            BiographyView {
                biography: id.to_string(),
                title: biography.title.clone(),
                start: biography.description.clone(),
                unlocked,
                unlock_hint: (!unlocked).then(|| biography.unlock_hint.clone()).flatten(),
            }
        })
        .collect()
}

/// Итоги: заметка, капитал — в рублях, а с порога правил в долларах, — лучшая и худшая сделки,
/// срок жизни, дела и открытия.
pub(super) fn summary(game: &Game, summary: &LifeSummary) -> LifeSummaryView {
    let content = &game.content;
    let ended = summary.ended;
    let currency = if ended >= content.life.capital_in_dollars_from {
        Currency::Dollar
    } else {
        Currency::Ruble
    };
    let capital = content.money.from_real(summary.capital, currency, ended);
    let deal = |record: Option<&DealRecord>| {
        record.map_or_else(
            || NOTHING.to_owned(),
            |record| {
                format!(
                    "{}, {}",
                    content.goods[&record.good].short_name,
                    record.date.year()
                )
            },
        )
    };
    let worst = summary
        .worst_deal
        .as_ref()
        .filter(|worst| summary.best_deal.as_ref() != Some(*worst));
    let businesses = if summary.businesses.is_empty() {
        NOTHING.to_owned()
    } else {
        summary.businesses.join(BUSINESS_SEPARATOR)
    };
    let line = |label: &str, value: String| SummaryLineView {
        label: label.to_owned(),
        value,
    };
    LifeSummaryView {
        source: summary.source.clone(),
        headline: summary.headline.clone(),
        text: summary.text.clone(),
        lines: vec![
            line(
                &content.endings[&summary.ending].capital_label,
                content.money.format_money(capital, ended),
            ),
            line(BEST_DEAL, deal(summary.best_deal.as_ref())),
            line(WORST_DEAL, deal(worst)),
            line(LIVED, lived(ended)),
            line(BUSINESSES, businesses),
        ],
        discoveries: summary
            .discoveries
            .iter()
            .map(|biography| {
                format!(
                    "{OPENED_BIOGRAPHY} «{}»",
                    content.biographies[biography].title
                )
            })
            .collect(),
    }
}

/// Сколько прожито с 1 января 1984 года: «13 лет 7 месяцев».
fn lived(ended: GameDate) -> String {
    let start = GameDate::START;
    let years = u32::try_from(ended.year() - start.year()).expect("жизнь кончается не раньше 1984");
    let months = years * MONTHS_IN_YEAR + ended.month() - start.month();
    if months == 0 {
        return LESS_THAN_A_MONTH.to_owned();
    }
    term(months)
}
