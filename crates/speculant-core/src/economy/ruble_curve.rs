use serde::Deserialize;

use super::Denomination;
use super::log_interpolation::log_interpolate;
use crate::time::GameDate;

/// Величина в рублях на ценнике по годам: индекс цен (рублей за рубль 1984 года) или курс валюты
/// (рублей за единицу). В данных значения после деноминации пишутся в новых рублях, как на ценнике
/// своего года; ядро переводит их в рубли до деноминации, поэтому кривая непрерывна через узел.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(transparent)]
pub struct RubleCurve {
    points: Vec<RublePoint>,
}

/// Опорная точка: рубли на ценнике в начале года.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RublePoint {
    pub year: i32,
    pub rubles: f64,
}

impl RubleCurve {
    pub fn points(&self) -> &[RublePoint] {
        &self.points
    }

    /// Рубли до деноминации на дату. Кривая начинается 1 января 1984 года — это проверяет загрузка.
    pub fn old_rubles_on(&self, date: GameDate, denomination: Denomination) -> f64 {
        log_interpolate(&self.points, date, |point| {
            let starts = GameDate::new_year(point.year)
                .expect("год опорной точки проверен загрузкой контента");
            let divisor = denomination.divisor_on(starts) as f64;
            (point.year, point.rubles * divisor)
        })
        .expect("рублёвая кривая начинается 1 января 1984 года")
    }
}
