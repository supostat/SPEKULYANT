use serde::{Deserialize, Serialize};

use super::Record;

/// Рекорды (решение 15.10): «Рейтинг Форбс» ironman-жизней и таблица свободного режима — каждая
/// от богатого к бедному, не длиннее размера из правил жизни.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Records {
    ironman: Vec<Record>,
    free: Vec<Record>,
}

impl Records {
    pub fn ironman(&self) -> &[Record] {
        &self.ironman
    }

    pub fn free(&self) -> &[Record] {
        &self.free
    }

    /// Жизнь встаёт на своё место в таблице своего режима; при равном капитале выше та, что
    /// раньше; хвост за пределом размера уходит.
    pub fn add(&mut self, record: Record, ironman: bool, size: usize) {
        let table = if ironman {
            &mut self.ironman
        } else {
            &mut self.free
        };
        table.push(record);
        table.sort_by(|left, right| right.capital.cmp(&left.capital));
        table.truncate(size);
    }
}
