use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use walkdir::WalkDir;

use crate::build_godot;
use crate::godot_check::{import_project, run_godot};
use crate::godot_pin;
use crate::project_root::project_root;

/// Прогоняет каждый скрипт `godot/src/**/*.gd` и `godot/tools/*.gd` через парсер Godot;
/// первая ошибка останавливает проверку с путём файла.
pub fn run() -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    let godot_dir = project_root().join("godot");
    let scripts = collect_scripts(&godot_dir)?;
    for script in &scripts {
        let resource_path = format!("res://{}", script.to_string_lossy().replace('\\', "/"));
        run_godot(
            &binary,
            &[
                "--headless",
                "--path",
                "godot",
                "--check-only",
                "--script",
                &resource_path,
            ],
        )
        .with_context(|| format!("gd-check: ошибка парсера в godot/{}", script.display()))?;
    }
    println!("gd-check: {} скриптов без ошибок парсера", scripts.len());
    Ok(())
}

fn collect_scripts(godot_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut scripts = Vec::new();
    for entry in WalkDir::new(godot_dir.join("src")).sort_by_file_name() {
        let entry = entry?;
        if is_gdscript(entry.path()) {
            scripts.push(relative_to(godot_dir, entry.path()));
        }
    }
    let tools_dir = godot_dir.join("tools");
    if tools_dir.is_dir() {
        let mut tool_scripts: Vec<PathBuf> = fs::read_dir(&tools_dir)?
            .map(|entry| entry.map(|entry| entry.path()))
            .collect::<Result<_, _>>()?;
        tool_scripts.sort();
        scripts.extend(
            tool_scripts
                .iter()
                .filter(|path| is_gdscript(path))
                .map(|path| relative_to(godot_dir, path)),
        );
    }
    if scripts.is_empty() {
        bail!("gd-check: в godot/src и godot/tools нет ни одного .gd");
    }
    Ok(scripts)
}

fn is_gdscript(path: &Path) -> bool {
    path.is_file() && path.extension().is_some_and(|extension| extension == "gd")
}

fn relative_to(godot_dir: &Path, path: &Path) -> PathBuf {
    path.strip_prefix(godot_dir)
        .expect("скрипт лежит внутри godot/")
        .to_path_buf()
}
