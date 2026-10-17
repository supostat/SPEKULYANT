use serde::{Deserialize, Serialize};

/// Шапка экрана: город, дата и деньги в кармане — каждая строка денег готова целиком
/// («Наличные: 1 240 р.», «$ 18 400», «Курс: 4 455 р.»).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeaderView {
    pub city: String,
    pub date: String,
    pub money: Vec<String>,
}
