//! CLI-симулятор баланса: боты проживают жизни на настоящем ядре и данных, отчёт — текстом в
//! вывод, а по флагам — JSON отчёта и покрытия событий для валидатора.

mod arguments;

use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::sync::Arc;
use std::{env, thread};

use serde::Serialize;
use speculant_core::Content;
use speculant_sim::{Report, simulate};

use arguments::{Arguments, USAGE};

/// Код выхода, когда командная строка не разобрана.
const USAGE_ERROR: u8 = 2;

fn main() -> ExitCode {
    let cores = thread::available_parallelism().map_or(1, usize::from);
    let arguments = match Arguments::parse(env::args().skip(1), cores) {
        Ok(arguments) => arguments,
        Err(error) => {
            eprintln!("{error}\n\n{USAGE}");
            return ExitCode::from(USAGE_ERROR);
        }
    };
    let content = match Content::load(&arguments.data) {
        Ok(content) => Arc::new(content),
        Err(errors) => {
            eprintln!(
                "{} не загружается (ошибок: {}) — сначала cargo run -p speculant-validate -- {}",
                arguments.data.display(),
                errors.len(),
                arguments.data.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let outcomes = simulate(&content, &arguments.settings);
    let report = Report::build(&content, arguments.settings.seed, &outcomes);
    print!("{report}");
    let written = arguments
        .report
        .as_deref()
        .map_or(Ok(()), |path| write_json(path, &report))
        .and_then(|()| {
            arguments
                .coverage
                .as_deref()
                .map_or(Ok(()), |path| write_json(path, &report.coverage))
        });
    match written {
        Ok(()) => ExitCode::SUCCESS,
        Err(failure) => {
            eprintln!("{failure}");
            ExitCode::FAILURE
        }
    }
}

fn write_json(path: &Path, value: &impl Serialize) -> Result<(), String> {
    let json = serde_json::to_string_pretty(value).expect("отчёт сериализуется в JSON");
    fs::write(path, json + "\n").map_err(|error| format!("не пишется {}: {error}", path.display()))
}
