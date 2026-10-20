//! Слухи о ценах в снимке (§5.2 п. 4): самые свежие дефициты и завалы других городов. Часть слухов
//! путает город; путает ли и какой город называет, решают сид мира и сам шок, поэтому слух не
//! меняется от снимка к снимку.

use super::super::Game;
use crate::content::CityId;
use crate::economy::Shock;
use crate::rng::{fnv1a, fraction_of};
use crate::time::GameDate;

const DEFICIT: &str = "дефицит";
const GLUT: &str = "завал";

/// Слухи о действующих шоках городов, кроме того, где игрок: «Рига: дефицит — Джинсы».
pub(super) fn price_rumors(game: &Game, today: GameDate) -> Vec<String> {
    let content = &game.content;
    game.world
        .markets
        .shocks()
        .iter()
        .rev()
        .filter(|shock| !shock.is_over_on(today))
        .filter_map(|shock| Some((shock, shock.city.as_ref()?)))
        .filter(|(_, city)| *city != &game.world.city)
        .take(content.swings.rumors)
        .map(|(shock, city)| {
            let kind = if shock.factor > 1.0 { DEFICIT } else { GLUT };
            let told = &content.cities[told_city(game, shock, city, today)].name;
            format!("{told}: {kind} — {}", content.goods[&shock.good].short_name)
        })
        .collect()
}

/// Город, который называет слух: верный — с долей точности правил, иначе другой открытый город.
fn told_city<'a>(game: &'a Game, shock: &Shock, city: &'a CityId, today: GameDate) -> &'a CityId {
    let until = shock.until.map_or(0, GameDate::day_index);
    let hash = fnv1a(
        game.world
            .rng
            .seed()
            .to_le_bytes()
            .into_iter()
            .chain(shock.good.as_str().bytes())
            .chain(city.as_str().bytes())
            .chain(until.to_le_bytes()),
    );
    if fraction_of(hash) < game.content.swings.rumor_accuracy {
        return city;
    }
    let others: Vec<&CityId> = game
        .content
        .cities
        .iter()
        .filter(|(id, other)| *id != city && other.is_open_on(today))
        .map(|(id, _)| id)
        .collect();
    let count = u64::try_from(others.len()).expect("городов мало");
    match count {
        0 => city,
        _ => others[usize::try_from(hash % count).expect("номер меньше числа городов")],
    }
}
