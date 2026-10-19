use crate::time::GameDate;

/// Значение на дату по опорным точкам «год → значение» в начале лет: между точками величина
/// меняется в одно и то же число раз за каждый день, после последней точки держится последнее
/// значение, до первой — `None`. Точки идут строго по возрастанию лет, значения больше нуля —
/// так их пропускает загрузка контента. Значение считается только у двух точек вокруг даты.
/// Логарифм и экспонента — из `libm`: одинаковый результат на всех платформах.
pub(crate) fn log_interpolate<P>(
    points: &[P],
    date: GameDate,
    year_of: impl Fn(&P) -> i32,
    value_of: impl Fn(&P) -> f64,
) -> Option<f64> {
    let starts = |point: &P| {
        GameDate::new_year(year_of(point)).expect("год опорной точки проверен загрузкой контента")
    };
    let year = date.year();
    let following = points.partition_point(|point| year_of(point) <= year);
    let from = points.get(following.checked_sub(1)?)?;
    let Some(next) = points.get(following) else {
        return Some(value_of(from));
    };
    let from_date = starts(from);
    if date == from_date {
        return Some(value_of(from));
    }
    let to_date = starts(next);
    let share = f64::from(date.day_index() - from_date.day_index())
        / f64::from(to_date.day_index() - from_date.day_index());
    Some(libm::exp(
        libm::log(value_of(from)) * (1.0 - share) + libm::log(value_of(next)) * share,
    ))
}
