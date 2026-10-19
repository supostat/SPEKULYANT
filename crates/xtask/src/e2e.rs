use anyhow::Result;

use crate::build_godot;
use crate::godot_check::{import_project, run_godot};
use crate::godot_pin;

const E2E_TOOL: &str = "res://tools/e2e_life.gd";

/// `cargo xtask e2e`: собирает мост и безголово проживает `godot/tools/e2e_life.gd` — полную
/// ironman-жизнь студента с сидом 7 на настоящих ядре, мосте, роутере и экранах: два месяца
/// торговли с событиями, поездка, выход в меню и «Продолжить» из слота с тем же снимком, ещё месяц,
/// уход на покой, итоги и запись в «Рейтинге Форбс». Подменён только каталог игрока — `user://e2e`.
/// Пиксели, звук и окно безголово не проверяются: их принимает человек по PNG, WAV и экспорту.
pub fn run() -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    run_godot(
        &binary,
        &["--headless", "--path", "godot", "--script", E2E_TOOL],
    )?;
    println!("e2e: полная жизнь прошла безголово на настоящих модулях — от заставки до итогов");
    Ok(())
}
