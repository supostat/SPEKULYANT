//! Мост gdext: тонкий слой между ядром `speculant-core` и Godot 4.
//! Класс `SpeculantGame` с четырьмя методами появится в фазе `bridge-contract`.

use godot::prelude::*;

struct SpeculantExtension;

#[gdextension]
unsafe impl ExtensionLibrary for SpeculantExtension {}

#[cfg(test)]
mod tests {
    const GDEXTENSION_MANIFEST: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../godot/speculant.gdextension"
    );

    #[test]
    fn gdextension_manifest_points_at_the_default_entry_symbol() {
        let manifest = std::fs::read_to_string(GDEXTENSION_MANIFEST)
            .expect("godot/speculant.gdextension читается");
        assert!(manifest.contains("entry_symbol = \"gdext_rust_init\""));
    }
}
