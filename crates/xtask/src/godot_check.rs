use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};

use crate::build_godot;
use crate::godot_pin;
use crate::project_root::project_root;

const ERROR_PREFIXES: [&str; 2] = ["ERROR:", "SCRIPT ERROR:"];
/// Сколько последних строк вывода показать, когда Godot упал: там его бэктрейс и паника моста.
const CRASH_TAIL_LINES: usize = 60;

/// Безголовый импорт проекта `godot/` пином Godot с собранным мостом: ресурсы импортируются,
/// расширение загружается, в выводе нет ни одной строки ошибки.
pub fn run() -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    println!("godot-check: проект импортируется безголово без ошибок");
    Ok(())
}

/// Импорт ресурсов и кэш глобальных классов: без него `preload` шрифта и `class_name` не разрешаются.
pub fn import_project(binary: &Path) -> Result<()> {
    run_godot(
        binary,
        &["--headless", "--path", "godot", "--import", "--quit"],
    )
}

/// Запускает Godot из корня репозитория. Godot возвращает 0 даже при ошибках скриптов и
/// расширений, поэтому провалом считается и ненулевой код, и строка ошибки в выводе. Если Godot
/// упал, к строкам ошибок добавляется хвост вывода: падение может не оставить ни одной.
pub fn run_godot(binary: &Path, arguments: &[&str]) -> Result<()> {
    let output = Command::new(binary)
        .args(arguments)
        .current_dir(project_root())
        .output()?;
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let error_lines = godot_errors(&combined);
    if !output.status.success() || !error_lines.is_empty() {
        let mut report = error_lines.join("\n");
        if !output.status.success() {
            let lines: Vec<&str> = combined.lines().collect();
            let tail = &lines[lines.len().saturating_sub(CRASH_TAIL_LINES)..];
            report.push_str(&format!("\nконец вывода Godot:\n{}", tail.join("\n")));
        }
        bail!(
            "Godot {arguments:?}: код {}, строк ошибок {}\n{report}",
            output.status,
            error_lines.len()
        );
    }
    Ok(())
}

/// Строки ошибок Godot в выводе или в файле лога.
pub fn godot_errors(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| {
            ERROR_PREFIXES
                .iter()
                .any(|prefix| line.trim_start().starts_with(prefix))
        })
        .collect()
}
