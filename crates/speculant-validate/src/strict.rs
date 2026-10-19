use std::collections::BTreeMap;

use speculant_core::Content;
use speculant_core::content::{ContentError, ContentFile, Event};
use speculant_core::npc::Strategy;

use crate::calendar::{Calendar, Narrowing};
use crate::chains;
use crate::days::Days;
use crate::locator::{Locator, event_path, quoted};
use crate::reachability::{Blocker, Reachability, Source};

/// Строгая проверка (§14.1, пункт 5): события, условия которых не выполнятся ни в одной жизни, —
/// окно вне дней эпохи или узла таймлайна, флаг, который ставят только недостижимые события или
/// ставят слишком поздно, зона, в которую никто не попадает, соперник без шаблона, — и мёртвые
/// цепочки продолжений. Ошибки — по возрастанию файла и строки.
pub fn check(content: &Content, files: &[ContentFile]) -> Vec<ContentError> {
    let locator = Locator::new(files);
    let calendar = Calendar::new(content);
    let mut errors = Vec::new();
    let mut days = BTreeMap::new();
    for (id, event) in &content.events {
        match static_days(content, &calendar, event) {
            Ok(event_days) => {
                days.insert(id.clone(), event_days);
            }
            Err((needle, reason)) => errors.push(locator.error(
                &event_path(id, event),
                Some(needle),
                format!("событие недостижимо: {reason}"),
            )),
        }
    }
    let reachability = Reachability::compute(content, &days);
    for (id, event_days) in &days {
        if reachability.earliest(id).is_some() {
            continue;
        }
        let event = &content.events[id];
        let blocker = reachability.blocker(event, event_days);
        errors.push(locator.error(
            &event_path(id, event),
            needle(&blocker).as_deref(),
            format!("событие недостижимо: {}", blocked(&blocker, event_days)),
        ));
    }
    errors.extend(chains::check(content, &days, &reachability, &locator));
    errors.sort();
    errors
}

/// Дни события по календарю — или ключ условия, которое их отрезало, и почему.
fn static_days(
    content: &Content,
    calendar: &Calendar<'_>,
    event: &Event,
) -> Result<Days, (&'static str, String)> {
    let days = calendar
        .event_days(event)
        .map_err(|narrowing: Narrowing| (narrowing.needle(), narrowing.message()))?;
    if let Some(strategy) = event.conditions.rival.and_then(|rival| rival.strategy)
        && !has_template(content, strategy)
    {
        return Err((
            "rival =",
            format!(
                "в npc/ нет ни одного шаблона соперника со стратегией «{}»",
                strategy.title()
            ),
        ));
    }
    Ok(days)
}

fn has_template(content: &Content, strategy: Strategy) -> bool {
    content
        .npcs
        .values()
        .any(|template| template.strategy == Some(strategy))
}

fn needle(blocker: &Blocker) -> Option<String> {
    let (Blocker::Missing(source) | Blocker::Late { source, .. }) = blocker;
    match source {
        Source::Flag(flag) => Some(quoted(flag.as_str())),
        Source::Zone => None,
    }
}

fn blocked(blocker: &Blocker, days: &Days) -> String {
    match blocker {
        Blocker::Missing(Source::Flag(flag)) => {
            format!("флаг {flag} ставят только недостижимые события")
        }
        Blocker::Missing(Source::Zone) => {
            "это событие зоны, а в зону не попасть: ни один достижимый выбор не даёт срока"
                .to_owned()
        }
        Blocker::Late {
            source: Source::Flag(flag),
            earliest,
        } => format!(
            "флаг {flag} ставится не раньше {}, а дни события — {days}",
            earliest.to_iso()
        ),
        Blocker::Late {
            source: Source::Zone,
            earliest,
        } => format!(
            "в зону попадают не раньше {}, а дни события — {days}",
            earliest.to_iso()
        ),
    }
}
