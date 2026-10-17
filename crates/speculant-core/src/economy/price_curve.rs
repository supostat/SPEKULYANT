use serde::Deserialize;

use super::Real;
use super::log_interpolation::log_interpolate;
use crate::time::GameDate;

/// Кривая цены товара по эпохам (§5.2 п. 1): цена в начале опорных лет в рублях 1984 года, между
/// точками цена меняется в одно и то же число раз за день. В данных — массив `curve` товара.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(transparent)]
pub struct PriceCurve {
    points: Vec<PricePoint>,
}

/// Опорная точка кривой: цена товара в начале года.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PricePoint {
    pub year: i32,
    pub price: Real,
}

impl PriceCurve {
    pub fn points(&self) -> &[PricePoint] {
        &self.points
    }

    /// Реальная цена на дату в копейках 1984 года; `None`, пока товара ещё нет — до первой точки.
    pub fn kopecks_on(&self, date: GameDate) -> Option<f64> {
        log_interpolate(&self.points, date, |point| {
            (point.year, point.price.kopecks() as f64)
        })
    }
}
