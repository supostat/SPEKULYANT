//! Строгая проверка: каждый вид дефекта ловится на своей негативной фикстуре, а исправный набор
//! проходит чисто.

mod overlay;

use speculant_validate::strict;

fn strict_errors(overlays: &[&str]) -> Vec<String> {
    let (content, files) = overlay::load(overlays);
    strict::check(&content, &files)
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn the_minimal_content_has_nothing_unreachable() {
    assert_eq!(strict_errors(&[]), Vec::<String>::new());
}

#[test]
fn a_window_outside_the_epoch_of_the_event_is_caught() {
    assert_eq!(
        strict_errors(&["two_epochs", "window_outside_epoch"]),
        [
            "events/stagnation/late_permit.toml:7: событие недостижимо: окно с 1990-01-01 до 1991-01-01 не пересекает его дни (с 1984-01-01 до 1988-01-01)"
        ]
    );
}

#[test]
fn an_epochs_condition_that_excludes_the_own_epoch_is_caught() {
    assert_eq!(
        strict_errors(&["two_epochs", "epochs_exclude_own"]),
        [
            "events/stagnation/cooperative_rumour.toml:7: событие недостижимо: условие epochs не пересекает его дни (с 1984-01-01 до 1988-01-01)"
        ]
    );
}

#[test]
fn a_window_that_ends_before_its_timeline_node_is_caught() {
    assert_eq!(
        strict_errors(&["before_timeline_node"]),
        [
            "events/stagnation/mmm_queue.toml:8: событие недостижимо: узел mmm случается не раньше 1984-07-01, а дни события — с 1984-01-01 до 1984-06-01"
        ]
    );
}

#[test]
fn a_flag_set_only_by_unreachable_events_is_caught() {
    assert_eq!(
        strict_errors(&["flag_from_unreachable"]),
        [
            "events/stagnation/deal_paid.toml:7: событие недостижимо: флаг secret_deal ставят только недостижимые события",
            "events/stagnation/secret_meeting.toml:8: событие недостижимо: узел mmm случается не раньше 1984-07-01, а дни события — с 1984-01-01 до 1984-06-01",
        ]
    );
}

#[test]
fn a_flag_set_after_the_last_day_of_the_event_is_caught() {
    assert_eq!(
        strict_errors(&["flag_set_too_late"]),
        [
            "events/stagnation/early_panic.toml:8: событие недостижимо: флаг mmm_collapsed ставится не раньше 1984-07-01, а дни события — с 1984-01-01 до 1984-06-01"
        ]
    );
}

#[test]
fn a_zone_nobody_is_sentenced_to_is_caught() {
    assert_eq!(
        strict_errors(&["zone_nobody_enters"]),
        [
            "zone/events/card_game.toml: событие недостижимо: это событие зоны, а в зону не попасть: ни один достижимый выбор не даёт срока"
        ]
    );
}

#[test]
fn a_sentence_opens_the_zone() {
    assert_eq!(
        strict_errors(&["zone_nobody_enters", "court_sentence"]),
        Vec::<String>::new()
    );
}

#[test]
fn a_rival_strategy_without_a_template_is_caught() {
    assert_eq!(
        strict_errors(&["rival_without_template"]),
        [
            "events/stagnation/racket_visit.toml:7: событие недостижимо: в npc/ нет ни одного шаблона соперника со стратегией «рэкетир»"
        ]
    );
}

#[test]
fn a_follow_up_whose_flags_the_choice_contradicts_is_caught() {
    assert_eq!(
        strict_errors(&["chain_flag_conflict"]),
        [
            "events/stagnation/loan.toml:14: выбор 1: мёртвая цепочка на debt_forgiven: оно требует, чтобы флага neighbour_owes не было, а выбор его ставит",
            "events/stagnation/loan.toml:26: выбор 2: мёртвая цепочка на debt_returned: оно требует флаг neighbour_owes, а выбор его снимает",
        ]
    );
}

#[test]
fn a_follow_up_that_lands_outside_its_days_is_caught() {
    assert_eq!(
        strict_errors(&["chain_out_of_dates"]),
        [
            "events/stagnation/summer_deal.toml:14: выбор 1: мёртвая цепочка на deal_result: через 30 дней оно приходится на дни с 1985-07-01 до 1985-10-01, а его дни — с 1990-01-01 до 1991-01-01"
        ]
    );
}

#[test]
fn a_zone_follow_up_without_a_sentence_is_caught() {
    assert_eq!(
        strict_errors(&["court_sentence", "chain_to_zone_without_sentence"]),
        [
            "events/stagnation/fake_arrest.toml:11: выбор 1: мёртвая цепочка на cell_talk: это событие зоны, а выбор не даёт срока"
        ]
    );
}

#[test]
fn a_node_that_starts_an_event_outside_its_days_is_caught() {
    assert_eq!(
        strict_errors(&["node_event_out_of_dates"]),
        [
            "history/mmm.toml:8: мёртвая цепочка на late_rally: узел случается в дни с 1984-07-01 до 1984-09-01, а дни события — с 1990-01-01 до 1991-01-01"
        ]
    );
}

#[test]
fn the_release_flag_needs_someone_sentenced() {
    assert_eq!(
        strict_errors(&["release_flag_event"]),
        [
            "events/stagnation/old_cellmate.toml:7: событие недостижимо: флаг released_from_zone ставят только недостижимые события"
        ]
    );
    assert_eq!(
        strict_errors(&["release_flag_event", "court_sentence"]),
        Vec::<String>::new()
    );
}

#[test]
fn a_sandbox_flag_comes_no_earlier_than_the_generator() {
    assert_eq!(
        strict_errors(&["template_flag_too_late"]),
        [
            "events/stagnation/shock_rumour.toml:8: событие недостижимо: флаг currency_shock ставится не раньше 2005-01-01, а дни события — с 1990-01-01 до 2000-01-01"
        ]
    );
}

#[test]
fn a_template_that_starts_an_event_outside_its_days_is_caught() {
    assert_eq!(
        strict_errors(&["template_event_out_of_dates"]),
        [
            "history/post2004/currency_shock.toml:10: мёртвая цепочка на exchange_queue: узел случается в дни с 2005-01-01 до 2061-01-01, а дни события — с 1990-01-01 до 2000-01-01"
        ]
    );
}
