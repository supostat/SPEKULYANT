use serde::{Deserialize, Serialize};

/// Рекорды (решение 15.10): «Рейтинг Форбс» ironman-жизней и таблица свободного режима.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordsView {
    pub ironman: Vec<RecordView>,
    pub free: Vec<RecordView>,
}

/// Строка рекордов: капитал — в рублях 1984 года, годы — «1984–1997».
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordView {
    pub name: String,
    pub biography: String,
    pub capital: String,
    pub years: String,
    pub ending: String,
}
