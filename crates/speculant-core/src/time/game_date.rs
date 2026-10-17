use serde::{Deserialize, Serialize};

const MONTHS_GENITIVE: [&str; 12] = [
    "января",
    "февраля",
    "марта",
    "апреля",
    "мая",
    "июня",
    "июля",
    "августа",
    "сентября",
    "октября",
    "ноября",
    "декабря",
];
const WEEKDAYS_SHORT: [&str; 7] = ["пн", "вт", "ср", "чт", "пт", "сб", "вс"];
const UNIX_DAY_OF_START: i64 = days_from_civil(1984, 1, 1);
const UNIX_EPOCH_WEEKDAY: i64 = 3;

/// День календаря как номер от 1 января 1984 года — первого дня каждой жизни. Даты раньше старта
/// (дни рождения персонажей) отрицательны.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct GameDate {
    day: i32,
}

impl GameDate {
    pub const START: Self = Self { day: 0 };

    pub fn from_day_index(day: i32) -> Self {
        Self { day }
    }

    pub fn day_index(self) -> i32 {
        self.day
    }

    /// Дата по году, месяцу и числу; `None` для несуществующей даты вроде 29 февраля 1989.
    pub fn from_ymd(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) || day == 0 {
            return None;
        }
        let unix_day = days_from_civil(i64::from(year), month, day);
        let date = Self {
            day: i32::try_from(unix_day - UNIX_DAY_OF_START).ok()?,
        };
        (date.year_month_day() == (year, month, day)).then_some(date)
    }

    /// Дата в записи ISO 8601 `ГГГГ-ММ-ДД`.
    pub fn parse_iso(text: &str) -> Option<Self> {
        let mut parts = text.splitn(3, '-');
        let year = parts.next()?.parse().ok()?;
        let month = parts.next()?.parse().ok()?;
        let day = parts.next()?.parse().ok()?;
        Self::from_ymd(year, month, day)
    }

    pub fn next(self) -> Self {
        Self { day: self.day + 1 }
    }

    pub fn year(self) -> i32 {
        self.year_month_day().0
    }

    pub fn month(self) -> u32 {
        self.year_month_day().1
    }

    pub fn day_of_month(self) -> u32 {
        self.year_month_day().2
    }

    /// Короткое русское имя дня недели: «пн» … «вс».
    pub fn weekday_short(self) -> &'static str {
        let index = (i64::from(self.day) + UNIX_DAY_OF_START + UNIX_EPOCH_WEEKDAY).rem_euclid(7);
        WEEKDAYS_SHORT[usize::try_from(index).expect("остаток от деления на 7 неотрицателен")]
    }

    /// Дата так, как её показывает шапка экрана: «14 марта 1986, пт».
    pub fn russian(self) -> String {
        let (year, month, day) = self.year_month_day();
        format!(
            "{day} {} {year}, {}",
            MONTHS_GENITIVE[month as usize - 1],
            self.weekday_short()
        )
    }

    /// Полных лет человеку, родившемуся `birth`, на эту дату.
    pub fn age_on(self, birth: Self) -> u32 {
        let (year, month, day) = self.year_month_day();
        let (birth_year, birth_month, birth_day) = birth.year_month_day();
        let before_birthday = (month, day) < (birth_month, birth_day);
        let age = year - birth_year - i32::from(before_birthday);
        u32::try_from(age).expect("день рождения не позже даты, на которую считается возраст")
    }

    fn year_month_day(self) -> (i32, u32, u32) {
        let (year, month, day) = civil_from_days(i64::from(self.day) + UNIX_DAY_OF_START);
        (
            i32::try_from(year).expect("год календаря игры укладывается в i32"),
            month,
            day,
        )
    }
}

/// Номер дня от 1970-01-01 по григорианскому календарю (алгоритм Говарда Хиннанта).
const fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let march_based_month = (month as i64 + 9) % 12;
    let day_of_year = (153 * march_based_month + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

fn civil_from_days(unix_day: i64) -> (i64, u32, u32) {
    let shifted = unix_day + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let march_based_month = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * march_based_month + 2) / 5 + 1;
    let month = if march_based_month < 10 {
        march_based_month + 3
    } else {
        march_based_month - 9
    };
    let year = year_of_era + era * 400 + i64::from(month <= 2);
    (
        year,
        u32::try_from(month).expect("месяц 1–12"),
        u32::try_from(day).expect("число 1–31"),
    )
}
