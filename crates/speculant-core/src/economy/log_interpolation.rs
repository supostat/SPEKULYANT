use crate::time::GameDate;

/// Значение на дату по опорным точкам «год → значение» в начале лет: между точками величина
/// меняется в одно и то же число раз за каждый день, после последней точки держится последнее
/// значение, до первой — `None`. Точки идут строго по возрастанию лет, значения больше нуля —
/// так их пропускает загрузка контента. Логарифм и экспонента — из `libm`: одинаковый результат
/// на всех платформах.
pub(crate) fn log_interpolate<P>(
    points: &[P],
    date: GameDate,
    year_and_value: impl Fn(&P) -> (i32, f64),
) -> Option<f64> {
    let dated = |point: &P| {
        let (year, value) = year_and_value(point);
        let starts =
            GameDate::new_year(year).expect("год опорной точки проверен загрузкой контента");
        (starts, value)
    };
    let following = points.partition_point(|point| dated(point).0 <= date);
    let (from_date, from_value) = dated(points.get(following.checked_sub(1)?)?);
    let Some(next) = points.get(following) else {
        return Some(from_value);
    };
    if date == from_date {
        return Some(from_value);
    }
    let (to_date, to_value) = dated(next);
    let share = f64::from(date.day_index() - from_date.day_index())
        / f64::from(to_date.day_index() - from_date.day_index());
    Some(libm::exp(
        libm::log(from_value) * (1.0 - share) + libm::log(to_value) * share,
    ))
}
