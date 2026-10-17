use std::fs;

use anyhow::{Context, Result, bail};

use crate::etalons;
use crate::screen_file::ScreenFile;
use crate::screen_runner;

/// Буфер экрана, нарисованного Godot безголово, посимвольно и по цветам совпадает с эталоном макета.
pub fn run(screen: &str) -> Result<()> {
    let actual = screen_runner::dump(screen)?;
    let etalon_path = etalons::etalon_path(screen);
    let expected = ScreenFile::parse(
        &fs::read_to_string(&etalon_path)
            .with_context(|| format!("нет эталона {}", etalon_path.display()))?,
    )?;
    if let Some(difference) = expected.first_difference(&actual) {
        bail!("screen-check {screen}: экран расходится с эталоном — {difference}");
    }
    println!("screen-check {screen}: буфер совпадает с эталоном посимвольно и по цветам");
    Ok(())
}
