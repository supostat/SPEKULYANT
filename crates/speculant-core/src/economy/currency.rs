use serde::{Deserialize, Serialize};

/// Валюта суммы (§3, §6). Рубль один на всю игру: до деноминации 1998 года это рубль СССР и
/// старый российский, после — новый; ядро держит рубли в копейках до деноминации, а ценник
/// показывает рубль своего дня. Чеки ВПТ «Берёзки» и доллары идут по курсу дня.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Currency {
    Ruble,
    Checks,
    Dollar,
}

impl Currency {
    /// Мелких единиц в единице: копеек в рубле и в чеке, центов в долларе.
    pub const MINOR_UNITS: i64 = 100;

    /// Число с обозначением валюты: «1 240 р.», «85 чек.», «$ 18 400».
    pub fn with_sign(self, number: &str) -> String {
        match self {
            Self::Ruble => format!("{number} р."),
            Self::Checks => format!("{number} чек."),
            Self::Dollar => format!("$ {number}"),
        }
    }
}
