use speculant_core::Content;
use speculant_core::content::{ContentError, ContentFile, EventId};
use speculant_sim::{Coverage, CoverageRules};

use crate::locator::{Locator, event_path};

/// Динамическое покрытие (§14.1, пункт 5): каждое событие хоть раз сработало за `[coverage] lives`
/// жизней прогона ботов или помечено `rare = true`. Покрытие должно быть снято с этих же данных и
/// не меньшим числом жизней, чем требует порог; `source` — откуда покрытие, для ошибок про него.
pub fn check(
    content: &Content,
    files: &[ContentFile],
    coverage: &Coverage,
    source: &str,
    rules: &CoverageRules,
) -> Vec<ContentError> {
    let locator = Locator::new(files);
    let mut errors = Vec::new();
    if coverage.lives < rules.lives.get() {
        errors.push(ContentError::new(
            source,
            None,
            format!(
                "покрытие снято за {} жизней, а [coverage] lives в balance.toml — {}",
                coverage.lives, rules.lives
            ),
        ));
    }
    let missing: Vec<&EventId> = content
        .events
        .keys()
        .filter(|event| !coverage.events.contains_key(*event))
        .collect();
    let unknown: Vec<&EventId> = coverage
        .events
        .keys()
        .filter(|event| !content.events.contains_key(*event))
        .collect();
    if !missing.is_empty() || !unknown.is_empty() {
        errors.push(ContentError::new(
            source,
            None,
            format!(
                "покрытие снято с других данных — в нём нет событий: {}; лишние: {}",
                listed(&missing),
                listed(&unknown)
            ),
        ));
    }
    for (id, event) in &content.events {
        if coverage.events.get(id) == Some(&0) && !event.rare {
            errors.push(locator.error(
                &event_path(id, event),
                Some("title ="),
                format!(
                    "событие ни разу не сработало за {} жизней прогона: поправь условия или пометь rare = true",
                    coverage.lives
                ),
            ));
        }
    }
    errors
}

/// Строка итога покрытия: сколько событий сработало и сколько редких не встретилось.
pub fn summary(content: &Content, coverage: &Coverage) -> String {
    let fired = coverage.events.values().filter(|count| **count > 0).count();
    let rare_unseen = coverage
        .events
        .iter()
        .filter(|(id, count)| {
            **count == 0 && content.events.get(*id).is_some_and(|event| event.rare)
        })
        .count();
    format!(
        "покрытие: за {} жизней сработали {fired} событий из {}, редких не встретилось {rare_unseen}",
        coverage.lives,
        content.events.len()
    )
}

fn listed(events: &[&EventId]) -> String {
    if events.is_empty() {
        return "—".to_owned();
    }
    events
        .iter()
        .map(|event| event.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}
