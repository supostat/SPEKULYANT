use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{Result, bail};
use walkdir::WalkDir;

use crate::font_glyphs;
use crate::project_root::project_root;

const NORMALIZATIONS: [(char, char); 2] = [('▶', '►'), ('▸', '►')];
const REPORTED_PROBLEMS: usize = 20;

/// Строковые литералы `godot/src/**/*.gd` и весь текст `data/**` состоят из глифов шрифта игры.
pub fn run() -> Result<()> {
    let glyphs = font_glyphs::load()?;
    let mut problems = Vec::new();
    let root = project_root();
    for path in files_with_extension(&root.join("godot").join("src"), Some("gd")) {
        let text = fs::read_to_string(&path)?;
        for (line_index, line) in text.lines().enumerate() {
            for literal in string_literals(line) {
                report_missing(&glyphs, &literal, &path, line_index, &mut problems);
            }
        }
    }
    for path in files_with_extension(&root.join("data"), None) {
        let text = fs::read_to_string(&path)?;
        for (line_index, line) in text.lines().enumerate() {
            report_missing(&glyphs, line, &path, line_index, &mut problems);
        }
    }
    if !problems.is_empty() {
        let shown: Vec<&String> = problems.iter().take(REPORTED_PROBLEMS).collect();
        bail!(
            "charset-check: {} символов вне шрифта игры\n{}",
            problems.len(),
            shown
                .iter()
                .map(|problem| problem.as_str())
                .collect::<Vec<_>>()
                .join("\n")
        );
    }
    println!("charset-check: все строки интерфейса из набора глифов шрифта");
    Ok(())
}

fn files_with_extension(dir: &Path, extension: Option<&str>) -> Vec<std::path::PathBuf> {
    if !dir.is_dir() {
        return Vec::new();
    }
    WalkDir::new(dir)
        .sort_by_file_name()
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .map(|entry| entry.into_path())
        .filter(|path| {
            extension.is_none_or(|wanted| path.extension().is_some_and(|found| found == wanted))
        })
        .collect()
}

fn report_missing(
    glyphs: &BTreeSet<char>,
    text: &str,
    path: &Path,
    line_index: usize,
    problems: &mut Vec<String>,
) {
    for character in text
        .chars()
        .filter(|character| *character != '\t' && !glyphs.contains(character))
    {
        let hint = NORMALIZATIONS
            .iter()
            .find(|(missing, _)| *missing == character)
            .map(|(_, replacement)| format!(", замени на «{replacement}»"))
            .unwrap_or_default();
        let relative = path.strip_prefix(project_root()).unwrap_or(path);
        problems.push(format!(
            "{}:{}: «{character}» (U+{:04X}) нет в шрифте{hint}",
            relative.display(),
            line_index + 1,
            u32::from(character)
        ));
    }
}

/// Содержимое строковых литералов GDScript в одной строке исходника; комментарий `#` вне строки обрывает разбор.
fn string_literals(line: &str) -> Vec<String> {
    let mut literals = Vec::new();
    let mut characters = line.chars();
    while let Some(character) = characters.next() {
        match character {
            '#' => break,
            '"' | '\'' => {
                let quote = character;
                let mut literal = String::new();
                while let Some(inner) = characters.next() {
                    match inner {
                        '\\' => {
                            literal.push(inner);
                            literal.extend(characters.next());
                        }
                        _ if inner == quote => break,
                        _ => literal.push(inner),
                    }
                }
                literals.push(literal);
            }
            _ => {}
        }
    }
    literals
}

#[cfg(test)]
mod tests {
    use super::string_literals;

    #[test]
    fn string_literals_skips_code_and_comments() {
        let line = r#"screen.put(1, 2, "Время идёт ►", 10)  # «не литерал» "и это тоже""#;
        assert_eq!(string_literals(line), vec!["Время идёт ►".to_owned()]);
    }

    #[test]
    fn string_literals_keeps_escaped_quotes_inside() {
        let line = r#"var path := 'C:\\GAMES\\SPEKUL' + "он сказал: \"нет\"""#;
        assert_eq!(
            string_literals(line),
            vec![
                r"C:\\GAMES\\SPEKUL".to_owned(),
                r#"он сказал: \"нет\""#.to_owned()
            ]
        );
    }
}
