use std::collections::BTreeMap;

/// Конфиг Godot — `project.godot`, `export_presets.cfg`: разделы `[раздел]` и строки
/// `ключ=значение`; строковое значение — в кавычках, они снимаются.
pub struct GodotConfig {
    sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl GodotConfig {
    pub fn parse(text: &str) -> Self {
        let mut sections: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
        let mut section = String::new();
        for line in text.lines().map(str::trim) {
            if line.is_empty() || line.starts_with(';') {
                continue;
            }
            if let Some(name) = line
                .strip_prefix('[')
                .and_then(|rest| rest.strip_suffix(']'))
            {
                section = name.to_owned();
                sections.entry(section.clone()).or_default();
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let value = value
                    .strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix('"'))
                    .unwrap_or(value);
                sections
                    .entry(section.clone())
                    .or_default()
                    .insert(key.to_owned(), value.to_owned());
            }
        }
        Self { sections }
    }

    pub fn value(&self, section: &str, key: &str) -> Option<&str> {
        self.sections.get(section)?.get(key).map(String::as_str)
    }

    /// Разделы пресетов экспорта `[preset.N]` по порядку.
    pub fn presets(&self) -> Vec<&str> {
        self.sections
            .keys()
            .map(String::as_str)
            .filter(|section| {
                section
                    .strip_prefix("preset.")
                    .is_some_and(|number| number.parse::<u32>().is_ok())
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::GodotConfig;

    const PRESETS: &str = "\
[preset.0]

name=\"Linux x86_64\"
runnable=true

[preset.0.options]

binary_format/architecture=\"x86_64\"
";

    #[test]
    fn sections_keys_and_unquoted_values_are_read() {
        let config = GodotConfig::parse(PRESETS);
        assert_eq!(config.value("preset.0", "name"), Some("Linux x86_64"));
        assert_eq!(config.value("preset.0", "runnable"), Some("true"));
        assert_eq!(
            config.value("preset.0.options", "binary_format/architecture"),
            Some("x86_64")
        );
        assert_eq!(config.presets(), ["preset.0"]);
    }
}
