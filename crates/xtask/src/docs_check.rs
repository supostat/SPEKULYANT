use std::collections::BTreeSet;
use std::fs;

use anyhow::{Context, Result, bail};

use crate::project_root::project_root;

const DOCUMENT: &str = "docs/SPEKULYANT.md";
const DESIGN_BOOK: &str = "design/design-book.html";
const MARKER_PREFIX: &str = "DECIDED-15-";
const OPEN_QUESTIONS_HEADING: &str = "## 15.";

/// Маркеры принятых решений `DECIDED-15-N` совпадают в документе и дизайн-книге;
/// с `--all-decided` каждый нумерованный пункт §15 документа обязан нести маркер.
pub fn run(require_all_decided: bool) -> Result<()> {
    let document = read(DOCUMENT)?;
    let design_book = read(DESIGN_BOOK)?;
    let decided_in_document = markers(&document);
    let decided_in_design_book = markers(&design_book);
    if decided_in_document != decided_in_design_book {
        bail!(
            "маркеры расходятся: только в {DOCUMENT} — {:?}, только в {DESIGN_BOOK} — {:?}",
            decided_in_document
                .difference(&decided_in_design_book)
                .collect::<Vec<_>>(),
            decided_in_design_book
                .difference(&decided_in_document)
                .collect::<Vec<_>>()
        );
    }
    if require_all_decided {
        let open = open_question_numbers(&document)?;
        let undecided: Vec<u32> = open.difference(&decided_in_document).copied().collect();
        if !undecided.is_empty() {
            bail!("в {DOCUMENT} §15 без решения остались вопросы {undecided:?}");
        }
    }
    println!(
        "docs-check: {} маркеров решений согласованы",
        decided_in_document.len()
    );
    Ok(())
}

fn read(relative: &str) -> Result<String> {
    let path = project_root().join(relative);
    fs::read_to_string(&path).with_context(|| format!("не читается {}", path.display()))
}

/// Номера N из всех вхождений `DECIDED-15-N` в тексте.
fn markers(text: &str) -> BTreeSet<u32> {
    text.match_indices(MARKER_PREFIX)
        .filter_map(|(index, _)| {
            let digits: String = text[index + MARKER_PREFIX.len()..]
                .chars()
                .take_while(char::is_ascii_digit)
                .collect();
            digits.parse().ok()
        })
        .collect()
}

/// Номера нумерованных пунктов раздела §15 документа.
fn open_question_numbers(document: &str) -> Result<BTreeSet<u32>> {
    let section = document
        .split_once(OPEN_QUESTIONS_HEADING)
        .map(|(_, rest)| rest)
        .with_context(|| format!("в {DOCUMENT} нет раздела «{OPEN_QUESTIONS_HEADING}»"))?;
    let section_body = section.split("\n## ").next().unwrap_or(section);
    Ok(section_body
        .lines()
        .filter_map(|line| {
            let (number, rest) = line.split_once(". ")?;
            (!rest.is_empty())
                .then(|| number.trim().parse().ok())
                .flatten()
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{markers, open_question_numbers};

    #[test]
    fn markers_collects_every_decided_number_once() {
        let text = "**Решено** (DECIDED-15-3) … DECIDED-15-11 … DECIDED-15-3 … DECIDED-15-x";
        assert_eq!(markers(text).into_iter().collect::<Vec<_>>(), vec![3, 11]);
    }

    #[test]
    fn open_question_numbers_reads_only_the_section_15_list() {
        let document = "## 14. Архитектура\n\n1. Не считается.\n\n## 15. Открытые вопросы\n\n1. Города.\n2. Товары.\n\n## 16. Хвост\n\n1. Тоже не считается.\n";
        assert_eq!(
            open_question_numbers(document)
                .unwrap()
                .into_iter()
                .collect::<Vec<_>>(),
            vec![1, 2]
        );
    }
}
