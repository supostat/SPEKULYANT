/// Сделанный выбор: его номер с нуля и строка того, что изменилось («Торговцы +1. Деньги −600 р.»).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Resolution {
    pub choice: usize,
    pub effects: String,
}
