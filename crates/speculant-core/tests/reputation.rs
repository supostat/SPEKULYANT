//! Репутация и связи (§9, решение 15.6): шесть фракций, силовики со стабилизации, уровни в
//! пределах правил с дрейфом к нулю в темпе эпохи, угроза эпохи в «Кармане» и люди, которых можно
//! попросить, которым можно задолжать и которых можно потерять.

use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{Content, EpochId, FactionId, NpcId};
use speculant_core::reputation::{Contact, ContactRefusal, FactionReputation, ReputationRules};
use speculant_core::time::GameDate;
use speculant_core::view::{ContactView, LevelView};
use speculant_core::{Game, LifeSettings};

const DATA: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../data");
const REPUTATION: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/reputation");
const RULES: ReputationRules = ReputationRules {
    limit: 7,
    max_debt: 2,
};
const DAY_MILLISECONDS: u32 = 1800;

fn load(dir: &str) -> Content {
    Content::load(Path::new(dir)).expect("контент валиден")
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

fn faction(id: &str) -> FactionId {
    FactionId::new(id)
}

fn new_game() -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed: 7,
    };
    Game::new_life(&settings, Arc::new(load(REPUTATION))).expect("жизнь начинается")
}

/// Двигает игру до даты по дню за раз.
fn advance_to(game: &mut Game, target: GameDate) {
    while game.today() < target {
        game.advance(DAY_MILLISECONDS);
    }
}

#[test]
fn the_six_factions_of_decision_15_6_are_in_the_data() {
    let content = load(DATA);
    let ids: Vec<&str> = content.factions.keys().map(FactionId::as_str).collect();
    assert_eq!(
        ids,
        vec![
            "bandits",
            "militia",
            "officials",
            "people",
            "siloviki",
            "traders"
        ]
    );
    let visible_on = |day: GameDate| -> Vec<&str> {
        content
            .factions
            .iter()
            .filter(|(_, faction)| faction.is_visible_on(day, &content.time.epochs))
            .map(|(id, _)| id.as_str())
            .collect()
    };
    assert!(!visible_on(date(1998, 12, 31)).contains(&"siloviki"));
    assert_eq!(visible_on(date(1998, 12, 31)).len(), 5);
    assert_eq!(
        visible_on(date(1999, 1, 1)).len(),
        6,
        "силовики со стабилизации"
    );
    let threats: Vec<(&str, &str, &str)> =
        ["stagnation", "perestroika", "wild_market", "stabilization"]
            .into_iter()
            .map(|epoch| {
                let threat = &content.epochs[&EpochId::new(epoch)].threat;
                (epoch, threat.faction.as_str(), threat.label.as_str())
            })
            .collect();
    assert_eq!(
        threats,
        vec![
            ("stagnation", "militia", "ОБХСС"),
            ("perestroika", "bandits", "Рэкет"),
            ("wild_market", "bandits", "Рэкет"),
            ("stabilization", "siloviki", "Силовики"),
        ]
    );
}

#[test]
fn a_reputation_level_stays_within_the_limit_and_hostility_is_its_negative_side() {
    let content = load(REPUTATION);
    let militia = faction("militia");
    let mut reputation = FactionReputation::new(&content.factions);
    assert_eq!(reputation.level(&militia), 0);
    reputation.change(&militia, 10, RULES);
    assert_eq!(reputation.level(&militia), 7);
    assert_eq!(
        reputation.hostility(&militia, RULES),
        0,
        "милиция к тебе добра"
    );
    reputation.change(&militia, -10, RULES);
    assert_eq!(reputation.level(&militia), -3);
    assert_eq!(reputation.hostility(&militia, RULES), 3);
    reputation.change(&militia, -100, RULES);
    assert_eq!(reputation.level(&militia), -7);
    assert_eq!(reputation.hostility(&militia, RULES), 7);
}

#[test]
fn reputation_drifts_toward_zero_at_the_pace_of_the_epoch() {
    let content = load(REPUTATION);
    let militia = faction("militia");
    let bandits = faction("bandits");
    let traders = faction("traders");
    assert_eq!(
        content
            .epoch_profile_on(date(1998, 12, 31))
            .reputation_drift_days,
        10
    );
    assert_eq!(
        content
            .epoch_profile_on(date(1999, 1, 1))
            .reputation_drift_days,
        5,
        "стабилизация забывает вдвое быстрее"
    );
    let mut reputation = FactionReputation::new(&content.factions);
    reputation.change(&militia, 3, RULES);
    reputation.change(&bandits, -2, RULES);
    for _ in 0..9 {
        reputation.next_day(10);
    }
    assert_eq!(
        (reputation.level(&militia), reputation.level(&bandits)),
        (3, -2)
    );
    reputation.next_day(10);
    assert_eq!(
        (reputation.level(&militia), reputation.level(&bandits)),
        (2, -1)
    );
    for _ in 0..40 {
        reputation.next_day(10);
    }
    assert_eq!(
        (
            reputation.level(&militia),
            reputation.level(&bandits),
            reputation.level(&traders)
        ),
        (0, 0, 0),
        "без поступков всё забывается, и ноль не уходит в минус"
    );
}

#[test]
fn the_pocket_shows_the_threat_of_the_epoch_and_the_count_of_contacts() {
    let mut game = new_game();
    let view = game.view();
    assert_eq!(
        view.pocket.danger,
        Some(LevelView {
            label: "ОБХСС".to_owned(),
            level: 0,
            scale: 7
        })
    );
    let contacts_line = view
        .pocket
        .items
        .last()
        .expect("в кармане есть строка связей");
    assert_eq!(
        (contacts_line.label.as_str(), contacts_line.amount.as_str()),
        ("Связей", "2")
    );
    let titles: Vec<String> = view
        .factions
        .iter()
        .map(|faction| faction.title.clone())
        .collect();
    assert_eq!(titles, vec!["Бандиты", "Милиция", "Торговцы"]);

    advance_to(&mut game, date(1999, 1, 1));
    let view = game.view();
    assert_eq!(
        view.pocket.danger.map(|danger| danger.label),
        Some("Силовики".to_owned()),
        "со стабилизацией угроза эпохи — силовики"
    );
    let titles: Vec<String> = view
        .factions
        .iter()
        .map(|faction| faction.title.clone())
        .collect();
    assert_eq!(titles, vec!["Бандиты", "Милиция", "Силовики", "Торговцы"]);
}

#[test]
fn the_starting_contacts_come_from_the_biography() {
    let game = new_game();
    assert_eq!(
        game.view().contacts,
        vec![
            ContactView {
                contact: "kofi".to_owned(),
                name: "Кофи".to_owned(),
                role: "сокурсник из Ганы".to_owned(),
                rival: false,
                relation: "знакомый".to_owned(),
                debt: None,
                availability: "на связи".to_owned(),
                history: Vec::new(),
            },
            ContactView {
                contact: "officer_syomin".to_owned(),
                name: "Участковый Сёмин".to_owned(),
                role: "участковый".to_owned(),
                rival: false,
                relation: "знакомый".to_owned(),
                debt: None,
                availability: "на связи".to_owned(),
                history: Vec::new(),
            },
        ]
    );
    assert_eq!(
        game.contacts()[&NpcId::new("kofi")],
        Contact::new(NpcId::new("kofi"))
    );
}

#[test]
fn a_contact_helps_until_too_many_favours_are_owed() {
    let today = date(1986, 3, 14);
    let mut kofi = Contact::new(NpcId::new("kofi"));
    assert_eq!(kofi.ask(RULES, today, "попросил достать джинсы"), Ok(()));
    assert_eq!(kofi.ask(RULES, today, "попросил продать пластинки"), Ok(()));
    assert_eq!(kofi.debt(), -2);
    assert_eq!(
        kofi.ask(RULES, today, "попросил ещё раз"),
        Err(ContactRefusal::OwedTooMuch)
    );
    kofi.help(1, today, "свёл с покупателем");
    assert_eq!(kofi.debt(), -1);
    assert_eq!(kofi.ask(RULES, today, "попросил подержать товар"), Ok(()));
    assert_eq!(
        kofi.history(),
        [
            "14.03.1986: попросил достать джинсы",
            "14.03.1986: попросил продать пластинки",
            "14.03.1986: свёл с покупателем",
            "14.03.1986: попросил подержать товар",
        ]
    );
    kofi.owe(3, today, "занял сто рублей");
    assert_eq!(
        kofi.debt(),
        -5,
        "задолжать можно и сверх того, что он стерпит"
    );
}

#[test]
fn an_offended_away_or_lost_contact_does_not_help() {
    let today = date(1986, 3, 14);
    let mut officer = Contact::new(NpcId::new("officer_syomin"));
    officer.change_relation(-1, RULES, today, "заметил ящик водки");
    assert_eq!(
        officer.ask(RULES, today, "попросил не замечать"),
        Err(ContactRefusal::Offended)
    );
    officer.change_relation(20, RULES, today, "получил коньяк к празднику");
    assert_eq!(officer.relation(), 7, "отношение в пределах правил");
    let back = date(1986, 4, 1);
    officer.send_away(back, today, "уехал на курсы");
    assert_eq!(
        officer.ask(RULES, date(1986, 3, 31), "позвонил"),
        Err(ContactRefusal::Away { until: back })
    );
    assert_eq!(
        officer.ask(RULES, back, "попросил предупредить о рейде"),
        Ok(())
    );
    officer.lose(date(1987, 1, 1), "перевели в другой район");
    assert!(officer.is_lost());
    assert_eq!(
        officer.ask(RULES, date(1987, 1, 2), "позвонил"),
        Err(ContactRefusal::Lost)
    );
    assert_eq!(
        ContactRefusal::Away { until: back }.to_string(),
        "не на связи до 01.04.1986"
    );
}
