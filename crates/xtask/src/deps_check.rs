use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;

use anyhow::{Context, Result, bail};
use cargo_metadata::{MetadataCommand, PackageId};
use serde::Deserialize;

use crate::project_root::project_root;
use crate::tools_manifest;

const CORE_CRATE: &str = "speculant-core";
const BRIDGE_CRATE: &str = "speculant-godot";
const GODOT_CRATE_PREFIX: &str = "godot";

/// Две инварианты зависимостей: ядро не тянет Godot ни прямо, ни транзитивно,
/// а пин gdext в манифесте моста совпадает с `tools.toml`.
pub fn run() -> Result<()> {
    check_core_is_free_of_godot()?;
    check_bridge_pin_matches_tools_manifest()?;
    println!("deps-check: ядро без Godot, пин gdext согласован с tools.toml");
    Ok(())
}

fn check_core_is_free_of_godot() -> Result<()> {
    let metadata = MetadataCommand::new()
        .manifest_path(project_root().join("Cargo.toml"))
        .exec()?;
    let resolve = metadata
        .resolve
        .context("cargo metadata без графа зависимостей")?;
    let names: HashMap<&PackageId, &str> = metadata
        .packages
        .iter()
        .map(|package| (&package.id, package.name.as_str()))
        .collect();
    let dependencies_of: HashMap<&PackageId, Vec<&PackageId>> = resolve
        .nodes
        .iter()
        .map(|node| (&node.id, node.deps.iter().map(|dep| &dep.pkg).collect()))
        .collect();
    let core_id = metadata
        .packages
        .iter()
        .find(|package| package.name.as_str() == CORE_CRATE)
        .map(|package| &package.id)
        .with_context(|| format!("в рабочей области нет крейта {CORE_CRATE}"))?;

    let mut visited: HashSet<&PackageId> = HashSet::new();
    let mut frontier = vec![core_id];
    let mut offenders = BTreeSet::new();
    while let Some(id) = frontier.pop() {
        if !visited.insert(id) {
            continue;
        }
        for dependency in dependencies_of.get(id).into_iter().flatten() {
            let name = names[dependency];
            if name.starts_with(GODOT_CRATE_PREFIX) {
                offenders.insert(name.to_owned());
            }
            frontier.push(dependency);
        }
    }
    if !offenders.is_empty() {
        bail!(
            "{CORE_CRATE} тянет Godot через зависимости: {}",
            offenders.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
    Ok(())
}

#[derive(Deserialize)]
struct BridgeManifest {
    dependencies: BridgeDependencies,
}

#[derive(Deserialize)]
struct BridgeDependencies {
    godot: GodotDependency,
}

#[derive(Deserialize)]
struct GodotDependency {
    version: String,
    features: Vec<String>,
}

fn check_bridge_pin_matches_tools_manifest() -> Result<()> {
    let pin = tools_manifest::load()?.gdext;
    let manifest_path = project_root()
        .join("crates")
        .join(BRIDGE_CRATE)
        .join("Cargo.toml");
    let manifest: BridgeManifest = toml::from_str(&fs::read_to_string(&manifest_path)?)
        .with_context(|| format!("{}: нет зависимости godot", manifest_path.display()))?;
    let godot = manifest.dependencies.godot;
    if godot.version != pin.version {
        bail!(
            "{BRIDGE_CRATE}: godot = {} в Cargo.toml, а tools.toml пинит {}",
            godot.version,
            pin.version
        );
    }
    if !godot.features.contains(&pin.api_feature) {
        bail!(
            "{BRIDGE_CRATE}: фича {} из tools.toml не включена у зависимости godot",
            pin.api_feature
        );
    }
    Ok(())
}
