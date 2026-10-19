use serde::Deserialize;

/// Сигареты и чай в пачках — валюта зоны (§12): запас при себе, цена услуги, содержимое передачи.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Stash {
    pub cigarettes: u32,
    pub tea: u32,
}

impl Stash {
    pub fn is_empty(self) -> bool {
        self == Self::default()
    }

    /// Хватает на цену.
    pub fn covers(self, price: Self) -> bool {
        self.cigarettes >= price.cigarettes && self.tea >= price.tea
    }

    pub fn plus(self, other: Self) -> Self {
        Self {
            cigarettes: self.cigarettes + other.cigarettes,
            tea: self.tea + other.tea,
        }
    }

    /// Остаток после платы; хватает ли, проверяет вызывающий.
    pub fn minus(self, price: Self) -> Self {
        assert!(self.covers(price), "платят только из того, что есть");
        Self {
            cigarettes: self.cigarettes - price.cigarettes,
            tea: self.tea - price.tea,
        }
    }
}
