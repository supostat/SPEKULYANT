//! Метапрогрессия (§11.3, решение 15.10): только знания и открытия, которые переходят из жизни в
//! жизнь, — открытые биографии, записная книжка, история, которую видели прошлые жизни (из неё
//! стартовые слухи), и рекорды, раздельно ironman и свободный режим.

mod notebook;
mod record;
mod records;

pub use notebook::Notebook;
pub use record::Record;
pub use records::Records;

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::content::{BiographyId, TimelineNodeId};

/// Всё, что игрок уносит из жизни в жизнь.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Meta {
    unlocked: BTreeSet<BiographyId>,
    notebook: Notebook,
    /// Исторические узлы, которые видела хоть одна жизнь, и год, когда видела последняя.
    known_history: BTreeMap<TimelineNodeId, i32>,
    records: Records,
}

impl Meta {
    /// Биографии, открытые прошлыми жизнями.
    pub fn unlocked(&self) -> &BTreeSet<BiographyId> {
        &self.unlocked
    }

    pub fn notebook(&self) -> &Notebook {
        &self.notebook
    }

    pub fn notebook_mut(&mut self) -> &mut Notebook {
        &mut self.notebook
    }

    pub fn known_history(&self) -> &BTreeMap<TimelineNodeId, i32> {
        &self.known_history
    }

    pub fn records(&self) -> &Records {
        &self.records
    }

    pub fn records_mut(&mut self) -> &mut Records {
        &mut self.records
    }

    pub fn unlock(&mut self, biographies: &[BiographyId]) {
        self.unlocked.extend(biographies.iter().cloned());
    }

    /// Исторический узел увидели в этом году.
    pub fn witness(&mut self, node: TimelineNodeId, year: i32) {
        self.known_history.insert(node, year);
    }
}
