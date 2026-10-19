use std::collections::BTreeMap;
use std::fs;

use anyhow::{Context, Result, anyhow, bail};
use serde::Deserialize;
use speculant_core::Content;

use crate::project_root::project_root;

const BALANCE_FILE: &str = "data/balance.toml";
const EVENTS_PREFIX: &str = "events:";

#[derive(Deserialize)]
struct Balance {
    volume: Volume,
}

/// Объём контента из `[volume]` (решения 15.1, 15.2 и 15.11).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Volume {
    goods: usize,
    cities: usize,
    npc: usize,
    zone_events: usize,
    post2004_templates: usize,
    events: BTreeMap<String, usize>,
}

/// `cargo xtask content-count <вид>`: сколько сущностей вида в `data/` — товаров, городов,
/// шаблонов соперников, событий эпохи (`events:<эпоха>`), событий зоны, шаблонов песочницы — и не
/// меньше ли их порога `[volume]` из `data/balance.toml`.
pub fn run(kind: &str) -> Result<()> {
    let root = project_root();
    let balance_path = root.join(BALANCE_FILE);
    let text = fs::read_to_string(&balance_path)
        .with_context(|| format!("не читается {}", balance_path.display()))?;
    let volume = toml::from_str::<Balance>(&text)
        .with_context(|| format!("{BALANCE_FILE} не разбирается"))?
        .volume;
    let content = Content::load(&root.join("data")).map_err(|errors| {
        anyhow!(
            "data/ не загружается ({} ошибок) — сначала cargo run -p speculant-validate -- data",
            errors.len()
        )
    })?;
    let (count, target) = match kind {
        "goods" => (content.goods.len(), volume.goods),
        "cities" => (content.cities.len(), volume.cities),
        "npc" => (
            content
                .npcs
                .values()
                .filter(|template| template.strategy.is_some())
                .count(),
            volume.npc,
        ),
        "zone-events" => (
            content
                .events
                .values()
                .filter(|event| event.epoch.is_none())
                .count(),
            volume.zone_events,
        ),
        "post2004-templates" => (content.history_templates.len(), volume.post2004_templates),
        _ => match kind.strip_prefix(EVENTS_PREFIX) {
            Some(epoch) => {
                let target = *volume.events.get(epoch).ok_or_else(|| {
                    anyhow!("для эпохи {epoch} нет порога в {BALANCE_FILE} [volume.events]")
                })?;
                let count = content
                    .events
                    .values()
                    .filter(|event| {
                        event
                            .epoch
                            .as_ref()
                            .is_some_and(|own| own.as_str() == epoch)
                    })
                    .count();
                (count, target)
            }
            None => bail!(
                "вид {kind} неизвестен: goods, cities, npc, events:<эпоха>, zone-events, post2004-templates"
            ),
        },
    };
    if count < target {
        bail!("content-count {kind}: {count} из {target}");
    }
    println!("content-count {kind}: {count} из {target}");
    Ok(())
}
