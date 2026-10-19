use serde::{Deserialize, Serialize};

use crate::time::GameDate;

/// Лента пейджера: сообщения дел, крыши, рынка и потерь, пришедшие, пока устройство было при
/// игроке, — последние, сколько держит лента.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pager {
    messages: Vec<PagerMessage>,
}

/// Сообщение: день, текст и срочность — потери и беды, которые ждут хозяина.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PagerMessage {
    pub date: GameDate,
    pub text: String,
    pub urgent: bool,
}

impl Pager {
    /// Кладёт сообщение в ленту; лента длиннее `length` теряет самые старые.
    pub fn record(&mut self, message: PagerMessage, length: usize) {
        self.messages.push(message);
        let excess = self.messages.len().saturating_sub(length);
        self.messages.drain(..excess);
    }

    /// Сообщения от старых к новым.
    pub fn messages(&self) -> &[PagerMessage] {
        &self.messages
    }

    /// Стирает сообщение — номер от нуля, от старых к новым; `false`, если такого нет.
    pub fn erase(&mut self, message: usize) -> bool {
        if message >= self.messages.len() {
            return false;
        }
        self.messages.remove(message);
        true
    }
}

#[cfg(test)]
mod tests {
    use super::{Pager, PagerMessage};
    use crate::time::GameDate;

    fn message(text: &str) -> PagerMessage {
        PagerMessage {
            date: GameDate::START,
            text: text.to_owned(),
            urgent: false,
        }
    }

    #[test]
    fn the_feed_keeps_only_the_latest_messages() {
        let mut pager = Pager::default();
        for text in ["раз", "два", "три"] {
            pager.record(message(text), 2);
        }
        let texts: Vec<&str> = pager
            .messages()
            .iter()
            .map(|message| message.text.as_str())
            .collect();
        assert_eq!(texts, ["два", "три"]);
    }

    #[test]
    fn an_erased_message_leaves_the_feed_and_a_missing_one_is_refused() {
        let mut pager = Pager::default();
        for text in ["раз", "два", "три"] {
            pager.record(message(text), 3);
        }
        assert!(pager.erase(1));
        assert!(!pager.erase(2), "сообщений осталось два");
        let texts: Vec<&str> = pager
            .messages()
            .iter()
            .map(|message| message.text.as_str())
            .collect();
        assert_eq!(texts, ["раз", "три"]);
    }
}
