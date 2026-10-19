use std::fmt;

use speculant_core::time::GameDate;

/// Отрезок дней: `from` входит, `until` — уже нет; без `until` отрезок бесконечен.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DaySpan {
    pub from: GameDate,
    pub until: Option<GameDate>,
}

impl DaySpan {
    pub fn new(from: GameDate, until: Option<GameDate>) -> Self {
        Self { from, until }
    }

    fn is_empty(self) -> bool {
        self.until.is_some_and(|until| until <= self.from)
    }

    fn intersect(self, other: Self) -> Self {
        let until = match (self.until, other.until) {
            (Some(left), Some(right)) => Some(left.min(right)),
            (left, right) => left.or(right),
        };
        Self::new(self.from.max(other.from), until)
    }
}

/// Дни, в которые что-то возможно: отрезки по возрастанию, не пересекаются.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Days {
    spans: Vec<DaySpan>,
}

impl Days {
    /// Все дни игры с её начала.
    pub fn all() -> Self {
        Self::from_spans([DaySpan::new(GameDate::START, None)])
    }

    pub fn from_spans(spans: impl IntoIterator<Item = DaySpan>) -> Self {
        let mut spans: Vec<DaySpan> = spans.into_iter().filter(|span| !span.is_empty()).collect();
        spans.sort_by_key(|span| span.from);
        Self { spans }
    }

    pub fn spans(&self) -> &[DaySpan] {
        &self.spans
    }

    pub fn is_empty(&self) -> bool {
        self.spans.is_empty()
    }

    /// Первый из дней.
    pub fn first(&self) -> Option<GameDate> {
        self.spans.first().map(|span| span.from)
    }

    pub fn intersect(&self, other: &Self) -> Self {
        Self::from_spans(
            self.spans
                .iter()
                .flat_map(|left| other.spans.iter().map(move |right| left.intersect(*right))),
        )
    }

    /// Те же дни, начиная с `day`.
    pub fn since(&self, day: GameDate) -> Self {
        self.intersect(&Self::from_spans([DaySpan::new(day, None)]))
    }

    /// Те же дни, сдвинутые на `days` вперёд.
    pub fn shifted(&self, days: u32) -> Self {
        Self::from_spans(self.spans.iter().map(|span| {
            DaySpan::new(
                span.from.after_days(days),
                span.until.map(|until| until.after_days(days)),
            )
        }))
    }
}

impl fmt::Display for Days {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.spans.is_empty() {
            return formatter.write_str("ни одного дня");
        }
        let shown: Vec<String> = self
            .spans
            .iter()
            .map(|span| match span.until {
                Some(until) => format!("с {} до {}", span.from.to_iso(), until.to_iso()),
                None => format!("с {}", span.from.to_iso()),
            })
            .collect();
        formatter.write_str(&shown.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use speculant_core::time::GameDate;

    use super::{DaySpan, Days};

    fn date(year: i32, month: u32, day: u32) -> GameDate {
        GameDate::from_ymd(year, month, day).expect("дата в календаре")
    }

    fn span(from: GameDate, until: GameDate) -> DaySpan {
        DaySpan::new(from, Some(until))
    }

    #[test]
    fn days_keep_only_what_both_sides_allow() {
        let epochs = Days::from_spans([
            span(date(1984, 1, 1), date(1988, 1, 1)),
            DaySpan::new(date(1999, 1, 1), None),
        ]);
        let window = Days::from_spans([span(date(1987, 6, 1), date(2000, 1, 1))]);
        assert_eq!(
            epochs.intersect(&window).to_string(),
            "с 1987-06-01 до 1988-01-01, с 1999-01-01 до 2000-01-01"
        );
    }

    #[test]
    fn days_that_do_not_meet_leave_nothing() {
        let stagnation = Days::from_spans([span(date(1984, 1, 1), date(1988, 1, 1))]);
        let later = Days::from_spans([span(date(1990, 1, 1), date(1991, 1, 1))]);
        let empty = stagnation.intersect(&later);
        assert!(empty.is_empty());
        assert_eq!(
            (empty.first(), empty.to_string()),
            (None, "ни одного дня".to_owned())
        );
    }

    #[test]
    fn a_shift_moves_both_ends_and_since_cuts_the_start() {
        let summer = Days::from_spans([span(date(1985, 6, 1), date(1985, 9, 1))]);
        assert_eq!(
            summer.shifted(30).since(date(1985, 8, 1)).to_string(),
            "с 1985-08-01 до 1985-10-01"
        );
    }
}
