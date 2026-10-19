//! Время мира: календарь от 1984 года, эпохи §3, часы решения 15.3 и подпотоки случая.

use rand_chacha::rand_core::Rng;
use speculant_core::rng::WorldRng;
use speculant_core::time::{Clock, FastForward, GameDate, TimeRules, TimeRulesError};

const TIME_TOML: &str = include_str!("../../../data/epochs/time.toml");

fn rules() -> TimeRules {
    TimeRules::from_toml(TIME_TOML).expect("data/epochs/time.toml валиден")
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn clock() -> Clock {
    Clock::new(rules().clock, GameDate::START)
}

#[test]
fn the_game_starts_on_sunday_the_first_of_january_1984() {
    assert_eq!(GameDate::START, date(1984, 1, 1));
    assert_eq!(GameDate::START.russian(), "1 января 1984, вс");
}

#[test]
fn the_market_mockup_date_formats_like_the_design_book() {
    assert_eq!(date(1986, 3, 14).russian(), "14 марта 1986, пт");
    assert_eq!(date(1998, 8, 17).russian(), "17 августа 1998, пн");
}

#[test]
fn twenty_years_of_play_hold_five_leap_days() {
    assert_eq!(date(2004, 1, 1).day_index(), 20 * 365 + 5);
    assert!(GameDate::from_ymd(1988, 2, 29).is_some());
    assert!(GameDate::from_ymd(1989, 2, 29).is_none());
    assert_eq!(date(1984, 12, 31).next(), date(1985, 1, 1));
}

#[test]
fn dates_before_the_game_are_negative_and_still_valid() {
    let birth = date(1961, 5, 10);
    assert!(birth.day_index() < 0);
    assert_eq!(birth.year(), 1961);
    assert_eq!(birth.month(), 5);
    assert_eq!(birth.day_of_month(), 10);
}

#[test]
fn age_turns_on_the_birthday() {
    let birth = date(1961, 5, 10);
    assert_eq!(date(1984, 5, 9).age_on(birth), 22);
    assert_eq!(date(1984, 5, 10).age_on(birth), 23);
}

#[test]
fn parse_iso_reads_only_valid_dates() {
    assert_eq!(GameDate::parse_iso("1991-08-19"), Some(date(1991, 8, 19)));
    assert_eq!(GameDate::parse_iso("1991-13-01"), None);
    assert_eq!(GameDate::parse_iso("19910819"), None);
}

#[test]
fn epochs_follow_the_frame_of_section_3() {
    let epochs = rules().epochs;
    let epoch_on = |year, month, day| epochs.epoch_on(date(year, month, day)).id.as_str();
    assert_eq!(epoch_on(1984, 1, 1), "stagnation");
    assert_eq!(epoch_on(1987, 12, 31), "stagnation");
    assert_eq!(epoch_on(1988, 1, 1), "perestroika");
    assert_eq!(epoch_on(1998, 8, 17), "wild_market");
    assert_eq!(epoch_on(2004, 12, 31), "stabilization");
    assert_eq!(epoch_on(2010, 6, 1), "sandbox");
}

#[test]
fn a_day_lasts_as_decision_15_3_says_on_each_speed() {
    let settings = rules().clock;
    let days = [1, 2, 3].map(|speed| settings.day_milliseconds_at(speed));
    assert_eq!(days, [1800, 1000, 500]);
    let mut clock = clock();
    assert_eq!(
        clock.advance(days[0]).len(),
        1,
        "на первой скорости — ровно день"
    );
}

#[test]
fn time_rules_refuse_broken_calendars() {
    let slower_second_speed = TIME_TOML.replace("[1800, 1000, 500]", "[1800, 2000, 500]");
    assert_eq!(
        TimeRules::from_toml(&slower_second_speed),
        Err(TimeRulesError::SpeedsNotAccelerating)
    );
    let two_speeds = TIME_TOML.replace("[1800, 1000, 500]", "[1800, 1000]");
    assert_eq!(
        TimeRules::from_toml(&two_speeds),
        Err(TimeRulesError::SpeedCount { found: 2 })
    );
    let late_start = TIME_TOML.replacen("starts = \"1984-01-01\"", "starts = \"1984-02-01\"", 1);
    assert!(matches!(
        TimeRules::from_toml(&late_start),
        Err(TimeRulesError::FirstEpochStart { .. })
    ));
    let unknown_key = TIME_TOML.replace("[clock]", "[clock]\nturbo = true");
    assert!(matches!(
        TimeRules::from_toml(&unknown_key),
        Err(TimeRulesError::Format(_))
    ));
}

#[test]
fn a_day_at_speed_one_takes_the_mockup_pace() {
    let mut clock = clock();
    assert!(clock.advance(1799).is_empty());
    let ticks = clock.advance(1);
    assert_eq!(ticks.len(), 1);
    assert_eq!(ticks[0].date, date(1984, 1, 2));
}

#[test]
fn faster_speeds_shorten_the_day() {
    let mut clock = clock();
    clock.set_speed(2).expect("скорость 2 есть");
    assert_eq!(clock.advance(1000).len(), 1);
    clock.set_speed(3).expect("скорость 3 есть");
    assert_eq!(clock.advance(1000).len(), 2);
}

#[test]
fn pause_stops_time_and_keeps_the_carry() {
    let mut clock = clock();
    assert!(clock.advance(900).is_empty());
    clock.set_paused(true);
    assert!(clock.advance(10_000).is_empty());
    clock.set_paused(false);
    assert_eq!(clock.advance(900).len(), 1);
}

#[test]
fn speeds_outside_one_to_three_are_refused() {
    let mut clock = clock();
    assert_eq!(clock.set_speed(0).map_err(|invalid| invalid.speed), Err(0));
    assert_eq!(clock.set_speed(4).map_err(|invalid| invalid.speed), Err(4));
    assert_eq!(clock.speed(), 1);
}

#[test]
fn fast_forward_runs_short_days_until_stopped_or_overridden() {
    let mut clock = clock();
    clock.start_fast_forward(FastForward::UntilEvent);
    assert_eq!(clock.advance(200).len(), 10);
    clock.stop_fast_forward();
    assert!(clock.advance(200).is_empty());
    clock.start_fast_forward(FastForward::UntilArrival);
    clock.set_speed(2).expect("скорость 2 есть");
    assert_eq!(clock.fast_forward(), None);
    clock.start_fast_forward(FastForward::UntilEvent);
    clock.set_paused(true);
    assert_eq!(clock.fast_forward(), None);
}

#[test]
fn many_days_in_one_advance_come_out_one_by_one() {
    let mut clock = clock();
    let ticks = clock.advance(1800 * 400);
    assert_eq!(ticks.len(), 400);
    for (offset, tick) in ticks.iter().enumerate() {
        assert_eq!(
            tick.date.day_index(),
            i32::try_from(offset).expect("смещение") + 1
        );
    }
}

#[test]
fn world_rng_streams_are_seeded_and_independent() {
    let draws = |rng: &mut WorldRng, stream: &str| -> Vec<u64> {
        (0..3).map(|_| rng.stream(stream).next_u64()).collect()
    };
    let mut quiet = WorldRng::new(42);
    let events_alone = draws(&mut quiet, "events");
    let mut busy = WorldRng::new(42);
    draws(&mut busy, "prices");
    draws(&mut busy, "prices");
    assert_eq!(
        draws(&mut busy, "events"),
        events_alone,
        "розыгрыши цен не сдвигают поток событий"
    );
    assert_ne!(
        draws(&mut WorldRng::new(42), "prices"),
        events_alone,
        "у подсистем разные потоки"
    );
    assert_ne!(
        draws(&mut WorldRng::new(43), "events"),
        events_alone,
        "другой сид — другая история"
    );
}
