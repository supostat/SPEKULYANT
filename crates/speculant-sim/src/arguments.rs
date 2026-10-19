use std::fmt;
use std::path::PathBuf;

use speculant_sim::{SimulationSettings, Strategy};

pub const USAGE: &str = "\
Использование: speculant-sim --lives <N> --seed <S> [--strategy <стратегия>] [--threads <N>]
                     [--data <каталог>] [--report <файл.json>] [--coverage <файл.json>]

  --lives      сколько жизней прожить
  --seed       сид прогона: из него — сиды всех жизней
  --strategy   одна стратегия вместо всех по кругу: random, arbitrage, hoarder, entrepreneur,
               cautious, reckless
  --threads    потоков (по умолчанию — сколько ядер)
  --data       каталог данных (по умолчанию data)
  --report     записать отчёт в JSON
  --coverage   записать покрытие событий в JSON для валидатора";

const DEFAULT_DATA: &str = "data";

/// Разобранная командная строка симулятора.
#[derive(Debug, PartialEq, Eq)]
pub struct Arguments {
    pub settings: SimulationSettings,
    pub data: PathBuf,
    pub report: Option<PathBuf>,
    pub coverage: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum ArgumentError {
    Missing { flag: &'static str },
    Unknown { argument: String },
    NoValue { flag: String },
    NotPositive { flag: String, value: String },
    UnknownStrategy { strategy: String },
}

impl Arguments {
    /// Флаги после имени программы; `cores` — потоков по умолчанию.
    pub fn parse(
        arguments: impl IntoIterator<Item = String>,
        cores: usize,
    ) -> Result<Self, ArgumentError> {
        let mut lives = None;
        let mut seed = None;
        let mut strategies = Strategy::ALL.to_vec();
        let mut threads = cores;
        let mut data = PathBuf::from(DEFAULT_DATA);
        let mut report = None;
        let mut coverage = None;
        let mut arguments = arguments.into_iter();
        while let Some(flag) = arguments.next() {
            let value = match flag.as_str() {
                "--lives" | "--seed" | "--strategy" | "--threads" | "--data" | "--report"
                | "--coverage" => arguments
                    .next()
                    .ok_or_else(|| ArgumentError::NoValue { flag: flag.clone() })?,
                _ => return Err(ArgumentError::Unknown { argument: flag }),
            };
            match flag.as_str() {
                "--lives" => lives = Some(positive(&flag, &value)?),
                "--seed" => seed = Some(number(&flag, &value)?),
                "--strategy" => {
                    let strategy = Strategy::parse(&value)
                        .ok_or(ArgumentError::UnknownStrategy { strategy: value })?;
                    strategies = vec![strategy];
                }
                "--threads" => threads = positive(&flag, &value)?,
                "--data" => data = PathBuf::from(value),
                "--report" => report = Some(PathBuf::from(value)),
                _ => coverage = Some(PathBuf::from(value)),
            }
        }
        Ok(Self {
            settings: SimulationSettings {
                lives: lives.ok_or(ArgumentError::Missing { flag: "--lives" })?,
                seed: seed.ok_or(ArgumentError::Missing { flag: "--seed" })?,
                strategies,
                threads,
            },
            data,
            report,
            coverage,
        })
    }
}

fn number(flag: &str, value: &str) -> Result<u64, ArgumentError> {
    value.parse().map_err(|_| ArgumentError::NotPositive {
        flag: flag.to_owned(),
        value: value.to_owned(),
    })
}

fn positive(flag: &str, value: &str) -> Result<usize, ArgumentError> {
    value
        .parse()
        .ok()
        .filter(|number| *number > 0)
        .ok_or_else(|| ArgumentError::NotPositive {
            flag: flag.to_owned(),
            value: value.to_owned(),
        })
}

impl fmt::Display for ArgumentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Missing { flag } => write!(formatter, "не задан {flag}"),
            Self::Unknown { argument } => write!(formatter, "неизвестный аргумент {argument}"),
            Self::NoValue { flag } => write!(formatter, "у {flag} нет значения"),
            Self::NotPositive { flag, value } => {
                write!(formatter, "{flag} {value}: нужно целое число больше нуля")
            }
            Self::UnknownStrategy { strategy } => {
                write!(formatter, "стратегии {strategy} нет")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CORES: usize = 8;

    fn parse(line: &str) -> Result<Arguments, ArgumentError> {
        Arguments::parse(line.split_whitespace().map(str::to_owned), CORES)
    }

    #[test]
    fn lives_and_seed_are_enough_for_a_run_of_every_strategy_on_every_core() {
        let arguments = parse("--lives 200 --seed 1").expect("строка верна");
        assert_eq!(
            arguments.settings,
            SimulationSettings {
                lives: 200,
                seed: 1,
                strategies: Strategy::ALL.to_vec(),
                threads: CORES,
            }
        );
        assert_eq!(arguments.data, PathBuf::from("data"));
        assert_eq!((arguments.report, arguments.coverage), (None, None));
    }

    #[test]
    fn every_flag_lands_in_its_place() {
        let arguments = parse(
            "--seed 9 --lives 12 --strategy hoarder --threads 2 --data other \
             --report report.json --coverage coverage.json",
        )
        .expect("строка верна");
        assert_eq!(arguments.settings.strategies, vec![Strategy::Hoarder]);
        assert_eq!(arguments.settings.threads, 2);
        assert_eq!(arguments.data, PathBuf::from("other"));
        assert_eq!(arguments.report, Some(PathBuf::from("report.json")));
        assert_eq!(arguments.coverage, Some(PathBuf::from("coverage.json")));
    }

    #[test]
    fn a_run_without_lives_or_seed_is_refused() {
        assert_eq!(
            parse("--seed 1").err(),
            Some(ArgumentError::Missing { flag: "--lives" })
        );
        assert_eq!(
            parse("--lives 3").err(),
            Some(ArgumentError::Missing { flag: "--seed" })
        );
    }

    #[test]
    fn bad_values_are_refused_by_name() {
        assert_eq!(
            parse("--lives 0 --seed 1").err(),
            Some(ArgumentError::NotPositive {
                flag: "--lives".to_owned(),
                value: "0".to_owned()
            })
        );
        assert_eq!(
            parse("--lives 1 --seed 1 --strategy gambler").err(),
            Some(ArgumentError::UnknownStrategy {
                strategy: "gambler".to_owned()
            })
        );
        assert_eq!(
            parse("--lives 1 --seed 1 --fast").err(),
            Some(ArgumentError::Unknown {
                argument: "--fast".to_owned()
            })
        );
        assert_eq!(
            parse("--lives 1 --seed").err(),
            Some(ArgumentError::NoValue {
                flag: "--seed".to_owned()
            })
        );
    }
}
