use anyhow::Result;

use crate::build_godot;
use crate::godot_check::{import_project, run_godot};
use crate::godot_pin;

const SMOKE_TOOL: &str = "res://tools/bridge_smoke.gd";

/// Собирает мост и безголово гоняет `godot/tools/bridge_smoke.gd`: класс `SpeculantGame` создаётся,
/// жизнь начинается, снимок разбирается, команда доходит до ядра. Любой провал — строка `ERROR:`.
pub fn run() -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    run_godot(
        &binary,
        &["--headless", "--path", "godot", "--script", SMOKE_TOOL],
    )?;
    println!("bridge-smoke: Godot создаёт SpeculantGame и читает снимок ядра");
    Ok(())
}
