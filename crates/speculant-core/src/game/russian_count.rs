//! Числа со словами по-русски: «1 точка», «3 года 6 месяцев», «12 пачек сигарет».

use crate::zone::Stash;

pub(super) const YEARS: [&str; 3] = ["год", "года", "лет"];
const MONTHS: [&str; 3] = ["месяц", "месяца", "месяцев"];
pub(super) const PACKS: [&str; 3] = ["пачка", "пачки", "пачек"];
pub(super) const MONTHS_IN_YEAR: u32 = 12;
const CIGARETTES: &str = "сигарет";
const TEA: &str = "чая";
const NOTHING: &str = "ничего";

/// Число со словом в нужной форме: «1 точка», «3 года», «13 лет». Формы — для 1, для 2–4 и для 5
/// и больше; 11–14 всегда берут третью.
pub(super) fn count(number: u32, forms: [&str; 3]) -> String {
    let [one, few, many] = forms;
    let word = match (number % 10, number % 100) {
        (_, 11..=14) => many,
        (1, _) => one,
        (2..=4, _) => few,
        _ => many,
    };
    format!("{number} {word}")
}

/// Срок в годах и месяцах: «3 года», «8 месяцев», «1 год 6 месяцев»; срок — от месяца.
pub(super) fn term(months: u32) -> String {
    match (months / MONTHS_IN_YEAR, months % MONTHS_IN_YEAR) {
        (0, months) => count(months, MONTHS),
        (years, 0) => count(years, YEARS),
        (years, months) => format!("{} {}", count(years, YEARS), count(months, MONTHS)),
    }
}

/// Сигареты и чай: «12 пачек сигарет, 3 пачки чая»; пустой запас — «ничего».
pub(super) fn packs(stash: Stash) -> String {
    let parts: Vec<String> = [(stash.cigarettes, CIGARETTES), (stash.tea, TEA)]
        .into_iter()
        .filter(|(number, _)| *number > 0)
        .map(|(number, what)| format!("{} {what}", count(number, PACKS)))
        .collect();
    if parts.is_empty() {
        return NOTHING.to_owned();
    }
    parts.join(", ")
}
