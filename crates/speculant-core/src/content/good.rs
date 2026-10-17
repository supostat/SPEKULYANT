use serde::Deserialize;

use crate::economy::Real;

/// Товар (§5.3): категория для множителей городов и кривая цены по эпохам в рублях 1984 года.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Good {
    pub name: String,
    pub category: String,
    pub unit: String,
    pub curve: Vec<PricePoint>,
}

/// Опорная точка кривой: цена товара в начале года.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PricePoint {
    pub year: i32,
    pub price: Real,
}
