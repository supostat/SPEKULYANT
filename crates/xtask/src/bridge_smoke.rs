use anyhow::Result;

use crate::build_godot;
use crate::godot_check::{import_project, run_godot};
use crate::godot_pin;

const SMOKE_TOOL: &str = "res://tools/bridge_smoke.gd";
const FULL_FLAG: &str = "--full";

/// Собирает мост и безголово гоняет `godot/tools/bridge_smoke.gd`: класс `SpeculantGame` создаётся,
/// жизнь начинается, снимок разбирается, команда доходит до ядра. С `full` — ещё тридцать дней с
/// событием и выбором, торговля, поездка, сохранение, загрузка и продолжение из слота и каждая
/// команда интерфейса. Любой провал — строка `ERROR:`.
pub fn run(full: bool) -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    let mut arguments = vec!["--headless", "--path", "godot", "--script", SMOKE_TOOL];
    if full {
        arguments.extend(["--", FULL_FLAG]);
    }
    run_godot(&binary, &arguments)?;
    if full {
        println!(
            "bridge-smoke --full: мост гоняет настоящее ядро по всем командам, сохранения в каталоге игрока"
        );
    } else {
        println!("bridge-smoke: Godot создаёт SpeculantGame и читает снимок ядра");
    }
    Ok(())
}
