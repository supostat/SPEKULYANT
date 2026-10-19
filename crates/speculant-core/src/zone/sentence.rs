use crate::time::GameDate;

/// Срок (§12): с какого дня и на сколько месяцев настоящего календаря. Суд может добавить месяцы,
/// амнистия — скостить.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Sentence {
    from: GameDate,
    months: u32,
}

impl Sentence {
    pub fn new(from: GameDate, months: u32) -> Self {
        Self { from, months }
    }

    pub fn from(&self) -> GameDate {
        self.from
    }

    pub fn months(&self) -> u32 {
        self.months
    }

    /// День выхода по сроку: то же число через срок, в коротком месяце — его последний день.
    pub fn until(&self) -> GameDate {
        self.from.after_months(self.months)
    }

    pub fn is_served_on(&self, today: GameDate) -> bool {
        today >= self.until()
    }

    /// День, к которому отсижена доля `share` срока.
    pub fn share_served_on(&self, share: f64) -> GameDate {
        let days = self.until().day_index() - self.from.day_index();
        let served = (f64::from(days) * share).ceil() as u32;
        self.from.after_days(served)
    }

    /// Кончилась ли сегодня неделя зоны: недели считаются от первого дня срока.
    pub fn is_week_end(&self, today: GameDate, week_days: u32) -> bool {
        let days = today.day_index() - self.from.day_index();
        days > 0 && days % i32::try_from(week_days).expect("дней в неделе зоны мало") == 0
    }

    /// Месяцы к сроку или из срока; короче нуля срок не бывает.
    pub fn change(&mut self, months: i32) {
        self.months = self.months.saturating_add_signed(months);
    }
}
