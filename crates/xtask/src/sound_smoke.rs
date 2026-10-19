use anyhow::Result;

use crate::build_godot;
use crate::godot_check::{import_project, run_godot};
use crate::godot_pin;
use crate::screen_runner;

const SOUND_TOOL: &str = "res://tools/sound_smoke.gd";
/// Мелодия и бипы для человека — в `.tools/out/pc_speaker.wav`.
const LISTENING_FILE: &str = "pc_speaker";

/// `cargo xtask sound-smoke`: собирает мост и безголово гоняет `godot/tools/sound_smoke.gd` —
/// пищалка PC-speaker выдаёт прямоугольную волну заданной частоты, пауза звучит тишиной, мелодия
/// заставки приходит из вшитых данных через снимок лобби, а узел Speaker заполняет буфер
/// AudioStreamGenerator. Мелодия и бипы ложатся в `.tools/out/pc_speaker.wav` — послушать их может
/// только человек. Любой провал — строка `ERROR:`.
pub fn run() -> Result<()> {
    let binary = godot_pin::fetch()?;
    build_godot::build_host_debug()?;
    import_project(&binary)?;
    let wav = screen_runner::output_path(LISTENING_FILE, "wav")?;
    let wav_path = wav.to_string_lossy();
    run_godot(
        &binary,
        &[
            "--headless",
            "--path",
            "godot",
            "--script",
            SOUND_TOOL,
            "--",
            &wav_path,
        ],
    )?;
    println!(
        "sound-smoke: прямоугольная волна нужной частоты, мелодия заставки из данных, Speaker; \
         послушать — {}",
        wav.display()
    );
    Ok(())
}
