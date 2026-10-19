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

/// Рисует экран макета безголово на его фикстуре и возвращает буфер из `.tools/out/<screen>.screen`.
pub fn dump(screen: &str) -> Result<ScreenFile> {
    let fixture = etalon_fixture(screen)?;
    dump_with(screen, fixture.as_deref())
}

/// Рисует экран безголово на снимке `StateView` из файла (без него — на пустом) и возвращает буфер.
pub fn dump_with(screen: &str, view: Option<&Path>) -> Result<ScreenFile> {
    let output = output_path(screen, "screen")?;
    run_screen_tool(DUMP_TOOL, true, screen, &output, view)?;
    ScreenFile::parse(&fs::read_to_string(&output)?)
}

/// Рисует экран макета на его фикстуре в окне и сохраняет PNG в `.tools/out/<screen>.png`.
pub fn screenshot(screen: &str) -> Result<PathBuf> {
    let fixture = etalon_fixture(screen)?;
    screenshot_with(screen, fixture.as_deref())
}

/// Рисует экран в окне на снимке `StateView` из файла и сохраняет PNG в `.tools/out/<screen>.png`.
pub fn screenshot_with(screen: &str, view: Option<&Path>) -> Result<PathBuf> {
    let output = output_path(screen, "png")?;
    run_screen_tool(SCREENSHOT_TOOL, false, screen, &output, view)?;
    Ok(output)
}

/// Файл в `.tools/out/` для экрана.
pub fn output_path(screen: &str, extension: &str) -> Result<PathBuf> {
    let dir = project_root().join(".tools").join("out");
    fs::create_dir_all(&dir)?;
    Ok(dir.join(format!("{screen}.{extension}")))
}

/// Фикстура `StateView` макета рядом с его эталоном, если она есть.
fn etalon_fixture(screen: &str) -> Result<Option<PathBuf>> {
    if !etalons::SCREENS.contains(&screen) {
        bail!(
            "у экрана «{screen}» нет макета, известны: {}",
            etalons::SCREENS.join(", ")
        );
    }
    let fixture = etalons::etalon_path(screen).with_extension("view.json");
    Ok(fixture.is_file().then_some(fixture))
}

/// Аргументы инструмента после `--`: имя экрана, выходной файл и снимок `StateView`, если он есть.
fn run_screen_tool(
    tool: &str,
    headless: bool,
    screen: &str,
    output: &Path,
    view: Option<&Path>,
) -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    let output = output.to_string_lossy().into_owned();
    let fixture = view.map(|path| path.to_string_lossy().into_owned());
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
