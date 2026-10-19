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
