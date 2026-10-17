use std::fs;

use anyhow::{Context, Result, bail};
use speculant_core::TimeRules;

use crate::project_root::project_root;

const TEMPLATE: &str = include_str!("new_event/template.toml");
const EPOCH_MARKER: &str = "__EPOCH__";

/// `cargo xtask new-event <эпоха> <имя>`: заготовка события в `data/events/<эпоха>/<имя>.toml`,
/// валидная для `speculant-validate`. Эпоха должна быть в календаре, существующий файл не затирается.
pub fn run(arguments: &[String]) -> Result<()> {
    let [epoch, name] = arguments else {
        bail!("использование: cargo xtask new-event <эпоха> <имя>");
    };
    let data = project_root().join("data");
    let calendar = data.join("epochs").join("time.toml");
    let rules = TimeRules::from_toml(&fs::read_to_string(&calendar)?)
        .with_context(|| format!("календарь {} не разбирается", calendar.display()))?;
    let epochs: Vec<&str> = rules
        .epochs
        .epochs()
        .iter()
        .map(|known| known.id.as_str())
        .collect();
    if !epochs.contains(&epoch.as_str()) {
        bail!(
            "эпохи «{epoch}» нет в календаре, есть: {}",
            epochs.join(", ")
        );
    }
    let is_identifier = name.starts_with(|first: char| first.is_ascii_lowercase())
        && name.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        });
    if !is_identifier {
        bail!("имя «{name}» — не идентификатор: латиница в нижнем регистре, цифры и _, с буквы");
    }
    let directory = data.join("events").join(epoch);
    let path = directory.join(format!("{name}.toml"));
    if path.exists() {
        bail!(
            "{} уже есть — заготовка не затирает событие",
            path.display()
        );
    }
    fs::create_dir_all(&directory)?;
    fs::write(&path, TEMPLATE.replace(EPOCH_MARKER, epoch))?;
    println!("Заготовка события: {}", path.display());
    Ok(())
}
