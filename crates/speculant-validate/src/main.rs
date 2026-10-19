//! CLI-валидатор контента: `speculant-validate <каталог data> [--strict] [--coverage <файл>]
//! [--watch]`. Печатает каждую ошибку как «файл:строка: сообщение»; с `--watch` перепроверяет
//! каталог при каждом сохранении файла (решение 15.7).

mod arguments;

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use speculant_core::Content;
use speculant_core::content::{ContentError, ContentFile};
use speculant_sim::{BalanceFile, Coverage};
use speculant_validate::{coverage, strict};

use crate::arguments::{Arguments, USAGE};

const SAVE_SETTLE: Duration = Duration::from_millis(200);

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let arguments = match Arguments::parse(&arguments) {
        Ok(arguments) => arguments,
        Err(message) => {
            eprintln!("{message}\n{USAGE}");
            return ExitCode::from(2);
        }
    };
    if arguments.watch {
        watch_directory(&arguments)
    } else {
        report(&arguments)
    }
}

fn report(arguments: &Arguments) -> ExitCode {
    let checked = Content::read_files(&arguments.directory).and_then(|files| {
        let content = Content::from_files(files.clone())?;
        let mut lines = vec![summary(&content)];
        let mut errors = Vec::new();
        if arguments.strict {
            errors.extend(strict::check(&content, &files));
            lines.push(format!(
                "строгая проверка: события достижимы, мёртвых цепочек нет — событий {}",
                content.events.len()
            ));
        }
        if let Some(path) = &arguments.coverage {
            match check_coverage(&content, &files, path, &arguments.directory) {
                Ok((found, line)) => {
                    errors.extend(found);
                    lines.push(line);
                }
                Err(error) => errors.push(error),
            }
        }
        if errors.is_empty() {
            Ok(lines)
        } else {
            Err(errors)
        }
    });
    match checked {
        Ok(lines) => {
            for line in lines {
                println!("{line}");
            }
            ExitCode::SUCCESS
        }
        Err(errors) => {
            for error in &errors {
                println!("{error}");
            }
            println!("ошибок: {}", errors.len());
            ExitCode::FAILURE
        }
    }
}

/// Сверка с покрытием прогона: ошибки и строка итога — или ошибка про само покрытие.
fn check_coverage(
    content: &Content,
    files: &[ContentFile],
    path: &Path,
    directory: &Path,
) -> Result<(Vec<ContentError>, String), ContentError> {
    let shown = path.display().to_string();
    let unreadable = |reason: String| ContentError::new(&shown, None, reason);
    let text =
        fs::read_to_string(path).map_err(|error| unreadable(format!("не читается: {error}")))?;
    let found: Coverage = serde_json::from_str(&text)
        .map_err(|error| unreadable(format!("не разбирается как покрытие: {error}")))?;
    let rules = BalanceFile::read(directory)
        .map_err(|error| unreadable(error.to_string()))?
        .coverage;
    Ok((
        coverage::check(content, files, &found, &shown, &rules),
        coverage::summary(content, &found),
    ))
}

fn summary(content: &Content) -> String {
    format!(
        "контент валиден: эпох {}, товаров {}, городов {}, маршрутов {}, мест хранения денег {}, фракций {}, биографий {}, жилья {}, финалов {}, людей {}, типов дел {}, форм собственности {}, политик {}, привычек {}, событий {}, услуг зоны {}, узлов истории {}, шаблонов песочницы {}",
        content.time.epochs.epochs().len(),
        content.goods.len(),
        content.cities.len(),
        content.routes.len(),
        content.money_places.len(),
        content.factions.len(),
        content.biographies.len(),
        content.housing.len(),
        content.endings.len(),
        content.npcs.len(),
        content.business_kinds.len(),
        content.business_forms.len(),
        content.policies.len(),
        content.habits.len(),
        content.events.len(),
        content.zone_services.len(),
        content.history.len(),
        content.history_templates.len()
    )
}

/// Проверяет каталог сразу и после каждой серии изменений; серия — изменения без паузы в 200 мс.
fn watch_directory(arguments: &Arguments) -> ExitCode {
    let directory = &arguments.directory;
    let (sender, receiver) = mpsc::channel();
    let mut watcher = match notify::recommended_watcher(sender) {
        Ok(watcher) => watcher,
        Err(error) => {
            eprintln!(
                "слежение за {} не запускается: {error}",
                directory.display()
            );
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = watcher.watch(directory, RecursiveMode::Recursive) {
        eprintln!(
            "слежение за {} не запускается: {error}",
            directory.display()
        );
        return ExitCode::FAILURE;
    }
    report(arguments);
    while receiver.recv().is_ok() {
        while receiver.recv_timeout(SAVE_SETTLE).is_ok() {}
        println!("— изменения в {}, проверяю заново —", directory.display());
        report(arguments);
    }
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use speculant_core::Content;

    use super::summary;

    const CONTENT_MIN: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../speculant-core/tests/fixtures/content-min"
    );

    #[test]
    fn summary_counts_every_kind_of_content() {
        let content = Content::load(Path::new(CONTENT_MIN)).expect("минимальный набор валиден");
        assert_eq!(
            summary(&content),
            "контент валиден: эпох 1, товаров 3, городов 2, маршрутов 2, мест хранения денег 1, фракций 3, биографий 1, жилья 1, финалов 2, людей 2, типов дел 1, форм собственности 1, политик 1, привычек 1, событий 3, услуг зоны 1, узлов истории 1, шаблонов песочницы 1"
        );
    }
}
