use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Result, bail};

use crate::build_godot;
use crate::godot_pin;
use crate::project_root::project_root;

const ERROR_PREFIXES: [&str; 2] = ["ERROR:", "SCRIPT ERROR:"];
/// Сколько последних строк вывода показать, когда Godot упал: там его бэктрейс и паника моста.
const CRASH_TAIL_LINES: usize = 60;
/// Список расширений проекта: Godot пишет его сам после первого скана, а в свежем checkout его ещё
/// нет. Записанный до импорта, он грузит мост при старте редактора, а не посреди первого скана, —
/// иначе безголовый редактор Godot 4.7 падает при выходе, не доделав документацию классов моста
/// (godotengine/godot#111048, #111645).
const EXTENSION_LIST: &str = "godot/.godot/extension_list.cfg";
const BRIDGE_MANIFEST: &str = "res://speculant.gdextension";

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
    list_bridge_extension()?;
    run_godot(
        binary,
        &["--headless", "--path", "godot", "--import", "--quit"],
    )
}

/// Мост в списке расширений проекта ещё до первого импорта; список, который Godot уже ведёт сам, не
/// трогается.
fn list_bridge_extension() -> Result<()> {
    let list = project_root().join(EXTENSION_LIST);
    if !list.is_file() {
        fs::create_dir_all(list.parent().expect("список лежит в каталоге .godot"))?;
        fs::write(&list, format!("{BRIDGE_MANIFEST}\n"))?;
    }
    Ok(())
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
