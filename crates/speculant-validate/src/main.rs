//! CLI-валидатор контента: `speculant-validate <каталог data> [--watch]`.
//! Печатает каждую ошибку как «файл:строка: сообщение»; с `--watch` перепроверяет каталог
//! при каждом сохранении файла (решение 15.7).

use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::mpsc;
use std::time::Duration;

use notify::{RecursiveMode, Watcher};
use speculant_core::Content;

const USAGE: &str = "использование: speculant-validate <каталог data> [--watch]";
const WATCH_FLAG: &str = "--watch";
const SAVE_SETTLE: Duration = Duration::from_millis(200);

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let watch = arguments.iter().any(|argument| argument == WATCH_FLAG);
    let directories: Vec<&String> = arguments
        .iter()
        .filter(|argument| *argument != WATCH_FLAG)
        .collect();
    let [directory] = directories.as_slice() else {
        eprintln!("{USAGE}");
        return ExitCode::from(2);
    };
    let directory = PathBuf::from(directory);
    if watch {
        watch_directory(&directory)
    } else {
        report(&directory)
    }
}

fn report(directory: &Path) -> ExitCode {
    match Content::load(directory) {
        Ok(content) => {
            println!("{}", summary(&content));
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
fn watch_directory(directory: &Path) -> ExitCode {
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
    report(directory);
    while receiver.recv().is_ok() {
        while receiver.recv_timeout(SAVE_SETTLE).is_ok() {}
        println!("— изменения в {}, проверяю заново —", directory.display());
        report(directory);
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
