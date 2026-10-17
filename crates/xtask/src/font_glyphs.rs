use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::project_root::project_root;

const FONT_FILE: &str = "PxPlus_IBM_VGA_9x16.ttf";
const GLYPHS_FILE: &str = "glyphs.txt";
const GLYPHS_PER_LINE: usize = 64;

fn fonts_dir() -> PathBuf {
    project_root().join("godot").join("assets").join("fonts")
}

/// Пишет `glyphs.txt` — все печатные символы, у которых в cmap шрифта есть глиф.
pub fn run() -> Result<()> {
    let glyphs = glyphs_of_font()?;
    let characters: Vec<char> = glyphs.into_iter().collect();
    let text: String = characters
        .chunks(GLYPHS_PER_LINE)
        .map(|line| line.iter().collect::<String>() + "\n")
        .collect();
    let path = fonts_dir().join(GLYPHS_FILE);
    fs::write(&path, text)?;
    println!(
        "font-glyphs: {} символов → {}",
        characters.len(),
        path.display()
    );
    Ok(())
}

/// Набор символов, которые умеет рисовать шрифт игры, из закоммиченного `glyphs.txt`.
pub fn load() -> Result<BTreeSet<char>> {
    let path = fonts_dir().join(GLYPHS_FILE);
    let text = fs::read_to_string(&path)
        .with_context(|| format!("нет {}: запусти cargo xtask font-glyphs", path.display()))?;
    Ok(text
        .chars()
        .filter(|character| *character != '\n')
        .collect())
}

fn glyphs_of_font() -> Result<BTreeSet<char>> {
    let path = fonts_dir().join(FONT_FILE);
    let data = fs::read(&path).with_context(|| format!("не читается {}", path.display()))?;
    let face = ttf_parser::Face::parse(&data, 0).context("шрифт не разбирается как TrueType")?;
    let cmap = face.tables().cmap.context("в шрифте нет таблицы cmap")?;
    let mut glyphs = BTreeSet::new();
    for subtable in cmap
        .subtables
        .into_iter()
        .filter(|subtable| subtable.is_unicode())
    {
        subtable.codepoints(|codepoint| {
            let has_glyph = subtable
                .glyph_index(codepoint)
                .is_some_and(|glyph| glyph.0 != 0);
            if let Some(character) =
                char::from_u32(codepoint).filter(|character| has_glyph && !character.is_control())
            {
                glyphs.insert(character);
            }
        });
    }
    if glyphs.is_empty() {
        bail!(
            "в unicode-таблицах cmap шрифта {} нет ни одного глифа",
            path.display()
        );
    }
    Ok(glyphs)
}
