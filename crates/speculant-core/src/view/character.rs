use serde::{Deserialize, Serialize};

/// Персонаж жизни: имя, возраст, здоровье словом состояния и жильё (решение 15.4).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CharacterView {
    pub name: String,
    /// «23 года».
    pub age: String,
    /// «здоров», «пошаливает сердце» — шкала скрыта.
    pub health: String,
    pub housing: String,
}

/// Жильё: где живёт персонаж и что можно купить сейчас, с ценой в номинале дня.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HomeView {
    pub housing: String,
    pub title: String,
    /// Цена в номинале дня; у жилья не в продаже её нет.
    pub price: Option<String>,
    pub current: bool,
    /// Можно купить сейчас: в продаже, хватает денег с зачётом прежнего, игрок в городе.
    pub available: bool,
}

/// Добровольный уход из меню: эмиграция, покой.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExitView {
    pub ending: String,
    pub title: String,
    pub available: bool,
}
