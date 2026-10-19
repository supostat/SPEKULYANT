use std::path::PathBuf;

pub const USAGE: &str = "использование: speculant-validate <каталог data> [--strict] \
                         [--coverage <покрытие.json>] [--watch]

  --strict     события, которые не случатся ни в одной жизни, и мёртвые цепочки продолжений
  --coverage   события без срабатываний в покрытии прогона speculant-sim --coverage
  --watch      перепроверять каталог при каждом сохранении файла";

/// Аргументы валидатора.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arguments {
    pub directory: PathBuf,
    pub strict: bool,
    pub coverage: Option<PathBuf>,
    pub watch: bool,
}

impl Arguments {
    pub fn parse(arguments: &[String]) -> Result<Self, String> {
        let mut directories = Vec::new();
        let mut strict = false;
        let mut coverage = None;
        let mut watch = false;
        let mut rest = arguments.iter();
        while let Some(argument) = rest.next() {
            match argument.as_str() {
                "--strict" => strict = true,
                "--watch" => watch = true,
                "--coverage" => {
                    let path = rest
                        .next()
                        .ok_or_else(|| "--coverage: нужен путь к покрытию".to_owned())?;
                    coverage = Some(PathBuf::from(path));
                }
                flag if flag.starts_with("--") => return Err(format!("неизвестный флаг {flag}")),
                directory => directories.push(PathBuf::from(directory)),
            }
        }
        let [directory] = <[PathBuf; 1]>::try_from(directories)
            .map_err(|_| "нужен ровно один каталог data".to_owned())?;
        Ok(Self {
            directory,
            strict,
            coverage,
            watch,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::Arguments;

    fn parse(line: &str) -> Result<Arguments, String> {
        let arguments: Vec<String> = line.split_whitespace().map(str::to_owned).collect();
        Arguments::parse(&arguments)
    }

    #[test]
    fn flags_go_in_any_order_around_the_directory() {
        assert_eq!(
            parse("--coverage target/coverage.json data --strict"),
            Ok(Arguments {
                directory: PathBuf::from("data"),
                strict: true,
                coverage: Some(PathBuf::from("target/coverage.json")),
                watch: false,
            })
        );
    }

    #[test]
    fn a_bare_directory_is_the_load_check() {
        assert_eq!(
            parse("data"),
            Ok(Arguments {
                directory: PathBuf::from("data"),
                strict: false,
                coverage: None,
                watch: false,
            })
        );
    }

    #[test]
    fn wrong_lines_are_refused() {
        assert_eq!(
            parse("data --coverage"),
            Err("--coverage: нужен путь к покрытию".to_owned())
        );
        assert_eq!(
            parse("data --fast"),
            Err("неизвестный флаг --fast".to_owned())
        );
        assert_eq!(
            parse("data other"),
            Err("нужен ровно один каталог data".to_owned())
        );
        assert_eq!(parse(""), Err("нужен ровно один каталог data".to_owned()));
    }
}
