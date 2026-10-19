//! Метапрогрессия в снимке (§11.3, решение 15.10): записная книжка, рекорды и стартовые слухи.
//! Файл метапрогрессии переживает обновления данных, поэтому записи о товарах, людях,
//! биографиях и финалах, которых в данных больше нет, не показываются.

use super::super::Game;
use crate::content::{CityId, Content};
use crate::economy::{Currency, Real};
use crate::life::PriceNote;
use crate::meta::Record;
use crate::time::GameDate;
use crate::view::{NotebookEntryView, RecordView, RecordsView};

const GOODS: &str = "Товары";
const PEOPLE: &str = "Люди";
const SCHEMES: &str = "Схемы";
const CHEAPEST_BUY: &str = "дешевле всего купил по";
const DEAREST_SELL: &str = "дороже всего продал по";
const BEST_DEAL: &str = "лучшая сделка:";
const IN_YEAR: &str = "году";
const PART_SEPARATOR: &str = "; ";
const RUMOR: &str = "Говорят, в";
/// Слух о том, что случится в этом году или в следующем.
const RUMOR_YEARS_AHEAD: i32 = 1;

/// Записи книжки: товары — почём брали и сбывали, люди — что о них знали, схемы — лучшие сделки.
pub(super) fn notebook(game: &Game) -> Vec<NotebookEntryView> {
    let content = &game.content;
    let notebook = game.meta.notebook();
    let entry = |topic: &str, title: &str, text: String| NotebookEntryView {
        topic: topic.to_owned(),
        title: title.to_owned(),
        text,
    };
    let goods = notebook.goods().iter().filter_map(|(good, prices)| {
        let name = &content.goods.get(good)?.name;
        let parts: Vec<String> = [
            (CHEAPEST_BUY, &prices.cheapest_buy),
            (DEAREST_SELL, &prices.dearest_sell),
        ]
        .into_iter()
        .filter_map(|(verb, note)| {
            let note = note.as_ref()?;
            Some(format!("{verb} {}", price_note(content, note)?))
        })
        .collect();
        Some(entry(GOODS, name, parts.join(PART_SEPARATOR)))
    });
    let people = notebook.people().iter().filter_map(|(npc, note)| {
        let person = content.npcs.get(npc)?;
        Some(entry(
            PEOPLE,
            &person.name,
            format!("{}. {note}", person.role),
        ))
    });
    let schemes = notebook.schemes().iter().filter_map(|(good, deal)| {
        let name = &content.goods.get(good)?.name;
        let profit = money_of(content, deal.profit, deal.date);
        Some(entry(
            SCHEMES,
            name,
            format!("{BEST_DEAL} +{profit} в {} {IN_YEAR}", deal.date.year()),
        ))
    });
    goods.chain(people).chain(schemes).collect()
}

/// Таблицы рекордов: капитал в рублях 1984 года.
pub(super) fn records(game: &Game) -> RecordsView {
    let content = &game.content;
    let rows = |table: &[Record]| -> Vec<RecordView> {
        table
            .iter()
            .filter_map(|record| {
                Some(RecordView {
                    name: record.name.clone(),
                    biography: content.biographies.get(&record.biography)?.title.clone(),
                    capital: money_of(content, record.capital, GameDate::START),
                    years: format!("{}–{}", GameDate::START.year(), record.ended.year()),
                    ending: content.endings.get(&record.ending)?.title.clone(),
                })
            })
            .collect()
    };
    let records = game.meta.records();
    RecordsView {
        ironman: rows(records.ironman()),
        free: rows(records.free()),
    }
}

/// Стартовые слухи (§11.3): исторические узлы, которые видели прошлые жизни, — если прошлая жизнь
/// видела их в этом году или в следующем, а в этом мире они ещё не случились. История плавает по
/// сиду, поэтому слух даёт преимущество, но не гарантию.
pub(super) fn rumors(game: &Game, today: GameDate) -> Vec<String> {
    let content = &game.content;
    let soon = today.year()..=today.year() + RUMOR_YEARS_AHEAD;
    let mut rumors: Vec<(i32, &str)> = game
        .meta
        .known_history()
        .iter()
        .filter(|(node, year)| {
            soon.contains(*year) && !game.world.timeline.has_happened(node, today)
        })
        .filter_map(|(node, year)| Some((*year, content.history.get(node)?.title.as_str())))
        .collect();
    rumors.sort();
    rumors
        .into_iter()
        .map(|(year, title)| format!("{RUMOR} {year} {IN_YEAR} — {title}."))
        .collect()
}

/// «140 р. (Москва, 1985)»; город, которого в данных нет, — повод запись не показывать.
fn price_note(content: &Content, note: &PriceNote) -> Option<String> {
    let city = city_name(content, &note.city)?;
    Some(format!(
        "{} ({city}, {})",
        money_of(content, note.price, note.date),
        note.date.year()
    ))
}

fn city_name<'a>(content: &'a Content, city: &CityId) -> Option<&'a str> {
    content.cities.get(city).map(|city| city.name.as_str())
}

/// Сумма в рублях 1984 года номиналом своего дня.
fn money_of(content: &Content, amount: Real, date: GameDate) -> String {
    let money = &content.money;
    money.format_money(money.from_real(amount, Currency::Ruble, date), date)
}
