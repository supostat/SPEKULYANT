use serde::Deserialize;

use crate::economy::Currency;
use crate::time::GameDate;

/// Место хранения денег (§6) из `data/money/<место>.toml`: валюта, окно, когда туда можно класть,
/// законность, годовой процент и риск, который читают игрок и события.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MoneyPlace {
    pub title: String,
    pub currency: Currency,
    /// С этой даты туда можно класть (коммерческий банк — с 1989).
    pub opens: Option<GameDate>,
    /// С этой даты туда класть нельзя (сберкасса — до 1992, чеки — до 1988).
    pub closes: Option<GameDate>,
    /// До этой даты хранение — статья (доллары до 1991).
    pub illegal_until: Option<GameDate>,
    /// Чем рискует вклад — строка для экрана денег.
    pub risk: String,
    /// Деньги лежат дома — «под матрасом»: их уносят кража и пожар, и риск зависит от жилья.
    #[serde(default)]
    pub home: bool,
    /// Годовой процент в валюте места с опорного года; до первой точки процентов нет.
    #[serde(default)]
    pub interest: Vec<InterestPoint>,
}

/// Годовой процент, действующий с начала года.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InterestPoint {
    pub year: i32,
    pub percent: f64,
}

impl MoneyPlace {
    pub fn accepts_deposits_on(&self, date: GameDate) -> bool {
        self.opens.is_none_or(|opens| date >= opens)
            && self.closes.is_none_or(|closes| date < closes)
    }

    pub fn is_legal_on(&self, date: GameDate) -> bool {
        self.illegal_until.is_none_or(|until| date >= until)
    }

    /// Годовой процент на дату: последняя опорная точка не позже года даты.
    pub fn annual_percent_on(&self, date: GameDate) -> f64 {
        self.interest
            .iter()
            .rev()
            .find(|point| point.year <= date.year())
            .map_or(0.0, |point| point.percent)
    }
}
