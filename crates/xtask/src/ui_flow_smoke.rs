use anyhow::Result;

use crate::build_godot;
use crate::godot_check::{import_project, run_godot};
use crate::godot_pin;

const FLOW_TOOL: &str = "res://tools/ui_flow.gd";

/// `cargo xtask ui-flow-smoke`: собирает мост и безголово гоняет `godot/tools/ui_flow.gd` —
/// клавиатурный сценарий на настоящем ядре в чистом каталоге игрока: заставка с мелодией → главное
/// меню → настройки → новая жизнь → рынок, бип отказа, покупка, пауза, событие с бипом, поездка →
/// дела, связи и пейджер без хода времени → сохранение → выход в главное меню → загрузка → ironman
/// → выход из игры, со сверкой буфера экрана. Любой провал — строка `ERROR:`.
pub fn run() -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    run_godot(
        &binary,
        &["--headless", "--path", "godot", "--script", FLOW_TOOL],
    )?;
    println!(
        "ui-flow-smoke: от заставки до выхода с клавиатуры — жизнь, экраны, сохранение и загрузка"
    );
    Ok(())
}
