use anyhow::{Result, bail};

pub const COLUMNS: usize = 80;
pub const ROWS: usize = 25;
const FOREGROUND_COLORS: u32 = 16;
const BACKGROUND_COLORS: u32 = 8;

/// Снимок экрана 80×25 в текстовом формате `.screen`: 25 строк символов, пустая строка,
/// 25 строк hex-цветов символов, пустая строка, 25 строк hex-цветов фона.
#[derive(Debug, PartialEq, Eq)]
pub struct ScreenFile {
    pub characters: Vec<String>,
    pub foreground: Vec<String>,
    pub background: Vec<String>,
}

impl ScreenFile {
    pub fn parse(text: &str) -> Result<Self> {
        let lines: Vec<&str> = text.lines().collect();
        let expected_lines = ROWS * 3 + 2;
        if lines.len() != expected_lines {
            bail!(
                "в .screen {} строк, а формат требует {expected_lines}",
                lines.len()
            );
        }
        if !lines[ROWS].is_empty() || !lines[ROWS * 2 + 1].is_empty() {
            bail!("слои .screen должны разделяться пустыми строками");
        }
        let layer = |start: usize| -> Vec<String> {
            lines[start..start + ROWS]
                .iter()
                .map(|line| (*line).to_owned())
                .collect()
        };
        let screen = Self {
            characters: layer(0),
            foreground: layer(ROWS + 1),
            background: layer(ROWS * 2 + 2),
        };
        screen.validate()?;
        Ok(screen)
    }

    pub fn validate(&self) -> Result<()> {
        for (row, line) in self.characters.iter().enumerate() {
            if line.chars().count() != COLUMNS {
                bail!(
                    "строка символов {row}: {} колонок вместо {COLUMNS}",
                    line.chars().count()
                );
            }
        }
        validate_color_layer("цвет символов", &self.foreground, FOREGROUND_COLORS)?;
        validate_color_layer("цвет фона", &self.background, BACKGROUND_COLORS)
    }

    pub fn render(&self) -> String {
        let mut text = String::new();
        for (index, layer) in [&self.characters, &self.foreground, &self.background]
            .iter()
            .enumerate()
        {
            if index > 0 {
                text.push('\n');
            }
            for line in layer.iter() {
                text.push_str(line);
                text.push('\n');
            }
        }
        text
    }

    /// Первое расхождение с другим снимком: слой, строка, колонка и обе строки целиком.
    pub fn first_difference(&self, actual: &Self) -> Option<String> {
        let layers = [
            ("символы", &self.characters, &actual.characters),
            ("цвет символов", &self.foreground, &actual.foreground),
            ("цвет фона", &self.background, &actual.background),
        ];
        for (layer_name, expected_rows, actual_rows) in layers {
            for (row, (expected, found)) in expected_rows.iter().zip(actual_rows.iter()).enumerate()
            {
                if expected != found {
                    let column = expected
                        .chars()
                        .zip(found.chars())
                        .position(|(left, right)| left != right)
                        .unwrap_or(COLUMNS);
                    return Some(format!(
                        "{layer_name}, строка {row}, колонка {column}\n  ожидалось: «{expected}»\n  получено:  «{found}»"
                    ));
                }
            }
        }
        None
    }
}

fn validate_color_layer(layer_name: &str, rows: &[String], colors: u32) -> Result<()> {
    for (row, line) in rows.iter().enumerate() {
        if line.chars().count() != COLUMNS {
            bail!(
                "{layer_name}, строка {row}: {} колонок вместо {COLUMNS}",
                line.chars().count()
            );
        }
        if let Some(digit) = line
            .chars()
            .find(|digit| digit.to_digit(16).is_none_or(|value| value >= colors))
        {
            bail!("{layer_name}, строка {row}: «{digit}» не цвет из диапазона 0..{colors}");
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{COLUMNS, ROWS, ScreenFile};

    fn blank_screen() -> ScreenFile {
        ScreenFile {
            characters: vec![" ".repeat(COLUMNS); ROWS],
            foreground: vec!["7".repeat(COLUMNS); ROWS],
            background: vec!["0".repeat(COLUMNS); ROWS],
        }
    }

    #[test]
    fn render_and_parse_round_trip() {
        let mut screen = blank_screen();
        screen.characters[3] = format!("{:<80}", "Проверка совести ... не найдена");
        screen.foreground[3] = "e".repeat(COLUMNS);
        let parsed = ScreenFile::parse(&screen.render()).expect("свой вывод разбирается");
        assert_eq!(parsed, screen);
    }

    #[test]
    fn parse_rejects_a_background_outside_eight_colors() {
        let mut screen = blank_screen();
        screen.background[0] = "8".repeat(COLUMNS);
        assert!(ScreenFile::parse(&screen.render()).is_err());
    }

    #[test]
    fn first_difference_names_layer_row_and_column() {
        let expected = blank_screen();
        let mut actual = blank_screen();
        actual.foreground[24] = format!("{}f{}", "7".repeat(5), "7".repeat(COLUMNS - 6));
        let difference = expected
            .first_difference(&actual)
            .expect("расхождение найдено");
        assert!(
            difference.starts_with("цвет символов, строка 24, колонка 5"),
            "{difference}"
        );
        assert!(expected.first_difference(&blank_screen()).is_none());
    }
}
