use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::thread;

use anyhow::{Context, Result, anyhow, bail};
use boa_engine::{Context as JsContext, Source};

use crate::project_root::project_root;
use crate::screen_file::{COLUMNS, ROWS, ScreenFile};

pub const SCREENS: [&str; 6] = ["boot", "market", "event", "manager", "pager", "end"];
const DESIGN_BOOK: &str = "design/design-book.html";
const ETALONS_DIR: &str = "design/etalons";
const PALETTE_FILE: &str = "palette.txt";
const EGA_SCRIPT: &str = "godot/src/text_screen/ega.gd";
const PALETTE_SIZE: usize = 16;
const DOM_SHIM: &str = include_str!("etalons/dom_shim.js");
/// Стек потока, где Boa исполняет скрипт дизайн-книги: разбор скрипта в отладочной сборке глубоко
/// рекурсивен — мегабайта, который Windows даёт главному потоку, не хватает, двух уже хватает на
/// macOS; здесь — с восьмикратным запасом.
const SCRIPT_STACK_BYTES: usize = 16 * 1024 * 1024;

pub fn etalon_path(screen: &str) -> PathBuf {
    project_root()
        .join(ETALONS_DIR)
        .join(format!("{screen}.screen"))
}

/// Исполняет скрипт дизайн-книги движком Boa и снимает начальные кадры шести макетов в `.screen`
/// и палитру EGA из CSS. С `check` ничего не пишет, а сверяет с закоммиченными файлами и `ega.gd`.
pub fn run(check: bool) -> Result<()> {
    let book = fs::read_to_string(project_root().join(DESIGN_BOOK))?;
    let mut generated: Vec<(PathBuf, String)> = Vec::new();
    for (screen, html) in screens_html(&book)? {
        let etalon = screen_from_html(&html).with_context(|| format!("макет scr-{screen}"))?;
        generated.push((etalon_path(&screen), etalon.render()));
    }
    let palette = palette_from_css(&book)?;
    ensure_ega_script_matches(&palette)?;
    generated.push((
        project_root().join(ETALONS_DIR).join(PALETTE_FILE),
        render_palette(&palette),
    ));
    if check {
        for (path, text) in &generated {
            let committed =
                fs::read_to_string(path).with_context(|| format!("нет {}", path.display()))?;
            if committed != *text {
                bail!(
                    "{} расходится с дизайн-книгой: запусти cargo xtask etalons",
                    path.display()
                );
            }
        }
        println!(
            "etalons --check: {} эталонов и палитра совпадают с дизайн-книгой",
            SCREENS.len()
        );
    } else {
        fs::create_dir_all(project_root().join(ETALONS_DIR))?;
        for (path, text) in &generated {
            fs::write(path, text)?;
        }
        println!(
            "etalons: записано {} файлов в {ETALONS_DIR}",
            generated.len()
        );
    }
    Ok(())
}

fn screens_html(book: &str) -> Result<BTreeMap<String, String>> {
    let script = book
        .split_once("<script>")
        .and_then(|(_, rest)| rest.split_once("</script>"))
        .map(|(script, _)| script)
        .context("в дизайн-книге нет блока <script>")?;
    let ids = serde_json::to_string(&SCREENS)?;
    let program = format!("{DOM_SHIM}\n{script}\n__etalonScreens({ids});");
    let json = thread::Builder::new()
        .stack_size(SCRIPT_STACK_BYTES)
        .spawn(move || evaluate(&program))?
        .join()
        .map_err(|_| anyhow!("поток Boa упал"))??;
    let screens: BTreeMap<String, String> = serde_json::from_str(&json)?;
    for screen in SCREENS {
        if screens.get(screen).is_none_or(String::is_empty) {
            bail!("скрипт дизайн-книги не нарисовал макет scr-{screen}");
        }
    }
    Ok(screens)
}

/// Исполняет программу движком Boa и отдаёт её результат строкой.
fn evaluate(program: &str) -> Result<String> {
    let mut context = JsContext::default();
    let value = context
        .eval(Source::from_bytes(program))
        .map_err(|error| anyhow!("скрипт дизайн-книги упал в Boa: {error}"))?;
    Ok(value
        .to_string(&mut context)
        .map_err(|error| anyhow!("Boa не отдал результат строкой: {error}"))?
        .to_std_string_escaped())
}

/// Разбирает вывод `Scr.html()` книги: строки через `\n`, в строке — span'ы с классами `bN fM`.
fn screen_from_html(html: &str) -> Result<ScreenFile> {
    let rows: Vec<&str> = html.split('\n').collect();
    if rows.len() != ROWS {
        bail!("{} строк вместо {ROWS}", rows.len());
    }
    let mut screen = ScreenFile {
        characters: Vec::new(),
        foreground: Vec::new(),
        background: Vec::new(),
    };
    for (row_index, row) in rows.iter().enumerate() {
        let (characters, foreground, background) =
            parse_row(row).with_context(|| format!("строка {row_index}"))?;
        screen.characters.push(characters);
        screen.foreground.push(foreground);
        screen.background.push(background);
    }
    screen.validate()?;
    Ok(screen)
}

fn parse_row(row: &str) -> Result<(String, String, String)> {
    const OPEN: &str = "<span class=\"";
    const CLOSE: &str = "</span>";
    let (mut characters, mut foreground, mut background) =
        (String::new(), String::new(), String::new());
    let mut rest = row;
    while !rest.is_empty() {
        let after_open = rest
            .strip_prefix(OPEN)
            .context("ожидался <span class=\"…\">")?;
        let (classes, after_classes) = after_open
            .split_once("\">")
            .context("незакрытый атрибут class")?;
        let (text, after_span) = after_classes.split_once(CLOSE).context("незакрытый span")?;
        let (background_color, foreground_color) = parse_classes(classes)?;
        for character in unescape(text).chars() {
            characters.push(character);
            foreground.push(hex_digit(foreground_color));
            background.push(hex_digit(background_color));
        }
        rest = after_span;
    }
    if characters.chars().count() != COLUMNS {
        bail!("{} колонок вместо {COLUMNS}", characters.chars().count());
    }
    Ok((characters, foreground, background))
}

fn parse_classes(classes: &str) -> Result<(u32, u32)> {
    let (background, foreground) = classes
        .split_once(' ')
        .with_context(|| format!("классы «{classes}» не в форме «bN fM»"))?;
    let number = |class: &str, prefix: char| -> Result<u32> {
        class
            .strip_prefix(prefix)
            .and_then(|digits| digits.parse().ok())
            .with_context(|| format!("класс «{class}» не в форме «{prefix}N»"))
    };
    Ok((number(background, 'b')?, number(foreground, 'f')?))
}

fn unescape(text: &str) -> String {
    text.replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
}

fn hex_digit(color: u32) -> char {
    char::from_digit(color, 16).expect("цвет EGA укладывается в одну hex-цифру")
}

/// Цвета `--e0`…`--e15` из CSS дизайн-книги в порядке индексов EGA.
fn palette_from_css(book: &str) -> Result<Vec<String>> {
    (0..PALETTE_SIZE)
        .map(|index| {
            let marker = format!("--e{index}:");
            let value = book
                .split_once(&marker)
                .and_then(|(_, rest)| rest.split_once(';'))
                .map(|(value, _)| value.trim().to_uppercase())
                .with_context(|| format!("в CSS дизайн-книги нет переменной {marker}"))?;
            Ok(value)
        })
        .collect()
}

fn render_palette(palette: &[String]) -> String {
    palette
        .iter()
        .enumerate()
        .map(|(index, color)| format!("{index} {color}\n"))
        .collect()
}

/// Константы `Color("#RRGGBB")` в `ega.gd` по порядку должны совпасть с палитрой книги.
fn ensure_ega_script_matches(palette: &[String]) -> Result<()> {
    let script = fs::read_to_string(project_root().join(EGA_SCRIPT))?;
    let colors: Vec<String> = script
        .match_indices("Color(\"")
        .filter_map(|(index, marker)| script[index + marker.len()..].split_once('"'))
        .map(|(color, _)| color.to_uppercase())
        .collect();
    if colors != palette {
        bail!("{EGA_SCRIPT}: цвета {colors:?} не совпадают с палитрой дизайн-книги {palette:?}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_row, screen_from_html};
    use crate::screen_file::{COLUMNS, ROWS};

    fn plain_row(class: &str) -> String {
        format!("<span class=\"{class}\">{}</span>", " ".repeat(COLUMNS))
    }

    #[test]
    fn parse_row_splits_spans_into_characters_and_colors() {
        let row = format!(
            "<span class=\"b0 f15\">1</span><span class=\"b3 f0\">Помощь &lt;&amp;&gt;</span><span class=\"b3 f0\">{}</span>",
            " ".repeat(COLUMNS - 11)
        );
        let (characters, foreground, background) = parse_row(&row).expect("строка разбирается");
        assert!(characters.starts_with("1Помощь <&>"));
        assert!(foreground.starts_with("f0000000000"));
        assert!(background.starts_with("03333333333"));
    }

    #[test]
    fn screen_from_html_requires_twenty_five_rows() {
        let rows = vec![plain_row("b1 f7"); ROWS - 1].join("\n");
        assert!(screen_from_html(&rows).is_err());
        let rows = vec![plain_row("b1 f7"); ROWS].join("\n");
        assert!(screen_from_html(&rows).is_ok());
    }
}
