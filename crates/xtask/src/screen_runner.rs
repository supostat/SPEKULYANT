use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};

use crate::build_godot;
use crate::etalons;
use crate::godot_check::{import_project, run_godot};
use crate::godot_pin;
use crate::project_root::project_root;
use crate::screen_file::ScreenFile;

const DUMP_TOOL: &str = "res://tools/dump_screen.gd";
const SCREENSHOT_TOOL: &str = "res://tools/screenshot.gd";

/// Рисует экран безголово и возвращает его буфер из `.tools/out/<screen>.screen`.
pub fn dump(screen: &str) -> Result<ScreenFile> {
    let output = output_path(screen, "screen")?;
    run_screen_tool(DUMP_TOOL, true, screen, &output)?;
    ScreenFile::parse(&fs::read_to_string(&output)?)
}

/// Рисует экран в окне и сохраняет PNG в `.tools/out/<screen>.png`.
pub fn screenshot(screen: &str) -> Result<PathBuf> {
    let output = output_path(screen, "png")?;
    run_screen_tool(SCREENSHOT_TOOL, false, screen, &output)?;
    Ok(output)
}

fn output_path(screen: &str, extension: &str) -> Result<PathBuf> {
    if !etalons::SCREENS.contains(&screen) {
        bail!(
            "неизвестный экран «{screen}», известны: {}",
            etalons::SCREENS.join(", ")
        );
    }
    let dir = project_root().join(".tools").join("out");
    fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{screen}.{extension}")))
}

/// Аргументы инструмента после `--`: имя экрана, выходной файл и фикстура `StateView`, если она есть.
fn run_screen_tool(tool: &str, headless: bool, screen: &str, output: &Path) -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    let output = output.to_string_lossy().into_owned();
    let fixture = etalons::etalon_path(screen).with_extension("view.json");
    let fixture = fixture
        .is_file()
        .then(|| fixture.to_string_lossy().into_owned());
    let mut arguments: Vec<&str> = Vec::new();
    if headless {
        arguments.push("--headless");
    }
    arguments.extend([
        "--path",
        "godot",
        "--script",
        tool,
        "--",
        screen,
        output.as_str(),
    ]);
    arguments.extend(fixture.as_deref());
    run_godot(&binary, &arguments)
}
