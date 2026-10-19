use std::collections::BTreeMap;

use speculant_core::Content;
use speculant_core::content::{
    Choice, Conditions, ContentError, Event, EventId, FollowUp, HistoryOutcome,
};

use crate::days::{DaySpan, Days};
use crate::locator::{Locator, event_path, node_path, quoted, template_path};
use crate::reachability::Reachability;

/// Мёртвые цепочки: продолжение, чьи условия несовместимы с выбором, который его назначил, или
/// которое выпадает из своих дней, и событие исхода узла, чьи дни не встречаются с окном узла.
/// События, недостижимые сами по себе, сюда не попадают: о них говорит проверка достижимости.
pub fn check(
    content: &Content,
    days: &BTreeMap<EventId, Days>,
    reachability: &Reachability,
    locator: &Locator<'_>,
) -> Vec<ContentError> {
    let mut errors = Vec::new();
    for (id, event_days) in days {
        let Some(earliest) = reachability.earliest(id) else {
            continue;
        };
        let event = &content.events[id];
        let path = event_path(id, event);
        let lived = event_days.since(earliest);
        for (index, choice) in event.choices.iter().enumerate() {
            for follow_up in &choice.follow_ups {
                let Some(next_days) = days.get(&follow_up.event) else {
                    continue;
                };
                let next = &content.events[&follow_up.event];
                if let Some(reason) = dead_end(event, choice, follow_up, &lived, next, next_days) {
                    errors.push(locator.error(
                        &path,
                        Some(&quoted(follow_up.event.as_str())),
                        format!(
                            "выбор {}: мёртвая цепочка на {}: {reason}",
                            index + 1,
                            follow_up.event
                        ),
                    ));
                }
            }
        }
    }
    for (node, history) in &content.history {
        let window = Days::from_spans([DaySpan::new(
            history.window.from,
            Some(history.window.until),
        )]);
        check_outcomes(
            &history.outcomes,
            &window,
            days,
            &node_path(node),
            locator,
            &mut errors,
        );
    }
    let generator = &content.history_generator;
    let sandbox = Days::from_spans([DaySpan::new(generator.from, Some(generator.until))]);
    for (template, data) in &content.history_templates {
        check_outcomes(
            &data.outcomes,
            &sandbox,
            days,
            &template_path(template),
            locator,
            &mut errors,
        );
    }
    errors
}

/// Почему продолжение `next` из выбора не начнётся ни в одной жизни: выбор на воле не даёт срока,
/// а это событие зоны; выбор противоречит его флагам; дни, прожитые событием (`lived`), со сдвигом
/// продолжения не встречаются с его днями.
fn dead_end(
    event: &Event,
    choice: &Choice,
    follow_up: &FollowUp,
    lived: &Days,
    next: &Event,
    next_days: &Days,
) -> Option<String> {
    let jails = choice.consequences.sentence_months.is_some();
    if event.epoch.is_some() && next.epoch.is_none() && !jails {
        return Some("это событие зоны, а выбор не даёт срока".to_owned());
    }
    flag_conflict(choice, &next.conditions).or_else(|| {
        let arrives = lived.shifted(follow_up.after_days);
        arrives.intersect(next_days).is_empty().then(|| {
            format!(
                "через {} дней оно приходится на дни {arrives}, а его дни — {next_days}",
                follow_up.after_days
            )
        })
    })
}

/// Выбор ставит флаг, которого продолжение не терпит, или снимает тот, которого оно ждёт.
fn flag_conflict(choice: &Choice, next: &Conditions) -> Option<String> {
    let consequences = &choice.consequences;
    if let Some(flag) = consequences
        .set_flags
        .iter()
        .find(|flag| next.flags_unset.contains(flag))
    {
        return Some(format!(
            "оно требует, чтобы флага {flag} не было, а выбор его ставит"
        ));
    }
    consequences
        .clear_flags
        .iter()
        .find(|flag| next.flags_set.contains(flag))
        .map(|flag| format!("оно требует флаг {flag}, а выбор его снимает"))
}

/// Событие исхода начинается в день узла — в его окне или в днях генератора.
fn check_outcomes(
    outcomes: &[HistoryOutcome],
    window: &Days,
    days: &BTreeMap<EventId, Days>,
    path: &str,
    locator: &Locator<'_>,
    errors: &mut Vec<ContentError>,
) {
    for outcome in outcomes {
        let Some(event) = &outcome.event else {
            continue;
        };
        let Some(event_days) = days.get(event) else {
            continue;
        };
        if window.intersect(event_days).is_empty() {
            errors.push(locator.error(
                path,
                Some(&quoted(event.as_str())),
                format!(
                    "мёртвая цепочка на {event}: узел случается в дни {window}, а дни события — {event_days}"
                ),
            ));
        }
    }
}
