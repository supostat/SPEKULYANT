//! История (§3, решение 15.9): таймлайн по сиду — даты плавают в окнах, рамки эпох стоят на
//! месте, масштаб и исходы меняются от сида к сиду; узел в свой день пишет новость, сдвигает цены,
//! ставит флаги и начинает своё событие; условие `timeline` ждёт узла; генератор песочницы
//! раскладывает шаблоны с заданной частотой, по весам и с отдыхом.

use std::collections::BTreeSet;
use std::path::Path;
use std::sync::Arc;

use speculant_core::content::{
    CityId, Content, ContentError, ContentFile, EventId, FlagId, GoodId, TimelineNodeId,
};
use speculant_core::history::TimelineNode;
use speculant_core::time::GameDate;
use speculant_core::{Command, CommandError, Game, LifeSettings};

const HISTORY: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/history");
const DAY_MILLISECONDS: u32 = 1800;

fn load() -> Arc<Content> {
    Arc::new(Content::load(Path::new(HISTORY)).expect("фикстура истории валидна"))
}

fn game_with(content: &Arc<Content>, seed: u64) -> Game {
    let settings = LifeSettings {
        biography: "student".to_owned(),
        ironman: true,
        seed,
    };
    Game::new_life(&settings, Arc::clone(content)).expect("жизнь начинается")
}

fn date(year: i32, month: u32, day: u32) -> GameDate {
    GameDate::from_ymd(year, month, day).expect("дата существует")
}

/// Узел таймлайна по имени; у исторического узла он один.
fn node<'a>(game: &'a Game, id: &str) -> &'a TimelineNode {
    game.timeline()
        .nodes()
        .iter()
        .find(|node| node.node.as_str() == id)
        .unwrap_or_else(|| panic!("узла {id} нет"))
}

fn historical<'a>(game: &'a Game, content: &Content) -> Vec<&'a TimelineNode> {
    game.timeline()
        .nodes()
        .iter()
        .filter(|node| content.history.contains_key(&node.node))
        .collect()
}

/// Живёт по дню до даты; событие узла читается первым выбором.
fn live_until(game: &mut Game, until: GameDate) {
    while game.today() < until {
        if game.events().pending().is_some() {
            game.apply(Command::Choose { choice: 1 })
                .expect("выбор события узла");
            game.apply(Command::Continue).expect("исход прочитан");
        }
        assert!(game.advance(DAY_MILLISECONDS), "время идёт");
    }
}

fn vodka_kopecks(game: &Game) -> f64 {
    game.markets()
        .normal_kopecks(
            game.content(),
            &CityId::new("moscow"),
            &GoodId::new("vodka"),
            game.today(),
        )
        .expect("водка в продаже")
}

#[test]
fn history_floats_inside_its_windows_by_seed() {
    let content = load();
    let mut dry_law_days = BTreeSet::new();
    let mut dry_law_scales = Vec::new();
    for seed in 1..=40 {
        let game = game_with(&content, seed);
        let nodes = historical(&game, &content);
        assert_eq!(nodes.len(), content.history.len(), "каждый узел — один раз");
        for node in nodes {
            let data = &content.history[&node.node];
            assert!(
                data.window.from <= node.date && node.date < data.window.until,
                "сид {seed}: {} {}",
                node.node,
                node.date.to_iso()
            );
            assert!(data.scale.from <= node.scale && node.scale <= data.scale.to);
        }
        let dry_law = node(&game, "dry_law");
        dry_law_days.insert(dry_law.date);
        dry_law_scales.push(dry_law.scale);
        assert_eq!(
            node(&game, "denomination").date,
            date(1990, 1, 1),
            "окно в один день не плавает"
        );
    }
    assert!(
        dry_law_days.len() >= 20,
        "сухой закон плавает по окну: {} разных дней",
        dry_law_days.len()
    );
    dry_law_scales.sort_by(f64::total_cmp);
    dry_law_scales.dedup();
    assert!(dry_law_scales.len() >= 20, "масштаб плавает по разбросу");
    assert_eq!(
        game_with(&content, 7).timeline(),
        game_with(&content, 7).timeline(),
        "один сид — одна история"
    );
    assert_ne!(
        game_with(&content, 7).timeline(),
        game_with(&content, 8).timeline()
    );
}

#[test]
fn the_epoch_frame_stays_while_history_floats() {
    let content = load();
    let epochs = &content.time.epochs;
    let starts: Vec<String> = epochs
        .epochs()
        .iter()
        .map(|epoch| epoch.starts.to_iso())
        .collect();
    assert_eq!(starts, ["1984-01-01", "1988-01-01", "2005-01-01"]);
    for seed in 1..=40 {
        let game = game_with(&content, seed);
        for node in historical(&game, &content) {
            let window = content.history[&node.node].window;
            assert_eq!(
                epochs.epoch_on(node.date).id,
                epochs.epoch_on(window.from).id,
                "сид {seed}: узел {} не переносит эпоху",
                node.node
            );
        }
    }
}

#[test]
fn alternatives_come_by_their_weights() {
    let content = load();
    let mut outcomes = [0; 2];
    for seed in 1..=200 {
        outcomes[node(&game_with(&content, seed), "mmm").outcome] += 1;
    }
    let [collapsed, lasted] = outcomes;
    assert!(lasted > 0, "пирамида иногда держится");
    assert!(
        collapsed > 2 * lasted,
        "рушится втрое чаще: {collapsed} против {lasted}"
    );
}

#[test]
fn a_node_strikes_on_its_day_with_its_news_scale_flags_and_event() {
    let content = load();
    let mut game = game_with(&content, 7);
    let dry_law = node(&game, "dry_law").clone();
    let news = "Указ о борьбе с пьянством: водку продают с двух часов дня.".to_owned();
    live_until(&mut game, dry_law.date.previous());
    assert!(!game.view().messages.contains(&news));
    assert!(!game.events().flags().contains(&FlagId::new("dry_law")));
    let before = vodka_kopecks(&game);

    assert!(game.advance(DAY_MILLISECONDS));
    assert_eq!(game.today(), dry_law.date);
    assert!(game.view().messages.contains(&news));
    assert!(game.events().flags().contains(&FlagId::new("dry_law")));
    let ratio = vodka_kopecks(&game) / before;
    assert!(
        (ratio - 2.0_f64.powf(dry_law.scale)).abs() < 1e-9,
        "шок ×2 в степени масштаба {}: {ratio}",
        dry_law.scale
    );
    assert_eq!(
        game.events()
            .pending()
            .map(|pending| pending.event.as_str()),
        Some("dry_law_queue"),
        "событие узла начинается в его день"
    );
    live_until(&mut game, dry_law.date.after_days(90));
    assert!(
        (vodka_kopecks(&game) - before).abs() < 1e-9,
        "через 90 дней шок кончился"
    );
}

#[test]
fn a_timeline_condition_waits_for_its_node_and_outcome() {
    let content = load();
    let mut game = game_with(&content, 7);
    live_until(&mut game, date(1989, 12, 31));
    let banknotes = EventId::new("new_banknotes");
    assert_eq!(
        game.start_event(&banknotes),
        Err(CommandError::EventNotReady {
            event: "new_banknotes".to_owned()
        }),
        "деноминации ещё не было"
    );
    live_until(&mut game, date(1990, 1, 1));
    game.start_event(&banknotes)
        .expect("в день деноминации узел уже случился");

    let seed_with = |outcome: usize| {
        (1..=200)
            .find(|seed| node(&game_with(&content, *seed), "mmm").outcome == outcome)
            .expect("за двести сидов встречается каждый исход")
    };
    let mut collapsed = game_with(&content, seed_with(0));
    let mmm = node(&collapsed, "mmm").date;
    live_until(&mut collapsed, mmm.previous());
    let victims = EventId::new("pyramid_victims");
    assert_eq!(
        collapsed.start_event(&victims),
        Err(CommandError::EventNotReady {
            event: "pyramid_victims".to_owned()
        }),
        "пирамида ещё не рухнула"
    );
    live_until(&mut collapsed, mmm);
    collapsed
        .start_event(&victims)
        .expect("после краха обманутые вкладчики у офиса");

    let mut lasted = game_with(&content, seed_with(1));
    let mmm = node(&lasted, "mmm").date;
    live_until(&mut lasted, mmm);
    assert!(
        lasted.start_event(&victims).is_err(),
        "пирамида устояла — вкладчиков нет"
    );
}

#[test]
fn the_sandbox_generator_keeps_its_frequency_weights_and_rests() {
    let content = load();
    let generator = content.history_generator;
    let span = generator.until.day_index() - generator.from.day_index();
    let shortest_rest = content
        .history_templates
        .values()
        .map(|template| template.rest_days)
        .min()
        .expect("шаблоны есть");
    let widest_gap = i32::try_from(shortest_rest + generator.gap_days.to).expect("промежуток мал");
    let mut counts = [0; 2];
    for seed in 1..=30 {
        let game = game_with(&content, seed);
        let generated: Vec<&TimelineNode> = game
            .timeline()
            .nodes()
            .iter()
            .filter(|node| content.history_templates.contains_key(&node.node))
            .collect();
        let count = i32::try_from(generated.len()).expect("узлов мало");
        assert!(
            span / widest_gap - 1 <= count && count <= span / 90 + 1,
            "сид {seed}: {count} узлов за {span} дней"
        );
        for pair in generated.windows(2) {
            assert!(
                pair[1].date.day_index() - pair[0].date.day_index() >= 90,
                "между узлами не меньше 90 дней"
            );
        }
        for (index, id) in ["currency_shock", "tech_wave"].into_iter().enumerate() {
            let id = TimelineNodeId::new(id);
            let template = &content.history_templates[&id];
            let dates: Vec<GameDate> = generated
                .iter()
                .filter(|node| node.node == id)
                .map(|node| node.date)
                .collect();
            for pair in dates.windows(2) {
                assert!(
                    pair[1] >= pair[0].after_days(template.rest_days),
                    "сид {seed}: {id} отдыхает {} дней",
                    template.rest_days
                );
            }
            counts[index] += dates.len();
        }
        assert!(
            generated
                .iter()
                .all(|node| generator.from < node.date && node.date < generator.until),
            "узлы генератора — между началом и горизонтом"
        );
    }
    let [shocks, waves] = counts;
    assert!(
        shocks > 2 * waves && waves > 0,
        "валютный шок тяжелее весом и короче отдыхом: {shocks} против {waves}"
    );
}

#[test]
fn the_world_lives_through_the_sandbox_nodes() {
    let content = load();
    let mut game = game_with(&content, 7);
    let first = game
        .timeline()
        .nodes()
        .iter()
        .find(|node| content.history_templates.contains_key(&node.node))
        .expect("генератор раскладывает узлы")
        .clone();
    live_until(&mut game, first.date);
    assert!(
        game.view().messages.contains(&first.outcome(&content).news),
        "узел песочницы пишет новость в свой день"
    );
}

/// Файлы фикстуры с правкой одного файла.
fn with_file(path: &str, edit: impl Fn(&str) -> String) -> Vec<ContentFile> {
    let mut files = Content::read_files(Path::new(HISTORY)).expect("файлы фикстуры читаются");
    let file = files
        .iter_mut()
        .find(|file| file.path == path)
        .unwrap_or_else(|| panic!("файла {path} нет"));
    file.text = edit(&file.text);
    files
}

/// Порча одного файла фикстуры и ошибка, которой загрузка на неё отвечает.
struct Defect {
    path: &'static str,
    edit: fn(&str) -> String,
    error: &'static str,
}

#[test]
fn the_loader_keeps_history_inside_its_frame() {
    let defects = [
        Defect {
            path: "history/dry_law.toml",
            edit: |text| {
                text.replace(
                    "window = { from = \"1985-04-01\", until = \"1985-08-01\" }",
                    "window = { from = \"1987-12-01\", until = \"1988-02-01\" }",
                )
            },
            error: "history/dry_law.toml:3: окно узла захватывает две эпохи — stagnation и perestroika: рамки эпох не плавают",
        },
        Defect {
            path: "history/generator.toml",
            edit: |text| text.replace("from = \"2005-01-01\"", "from = \"1990-03-01\""),
            error: "history/mmm.toml:3: окно узла заходит за начало генератора песочницы 1990-03-01",
        },
        Defect {
            path: "history/dry_law.toml",
            edit: |text| {
                text.replace(
                    "scale = { from = 0.5, to = 1.5 }",
                    "scale = { from = 1.5, to = 0.5 }",
                )
            },
            error: "history/dry_law.toml:4: масштаб от 1.5 до 0.5: больше нуля, и from не больше to",
        },
        Defect {
            path: "history/mmm.toml",
            edit: |text| text.replace("weight = 1", "weight = 0"),
            error: "history/mmm.toml:10: исход 2: вес больше нуля",
        },
        Defect {
            path: "history/dry_law.toml",
            edit: |text| text.replace("event = \"dry_law_queue\"", "event = \"dry_law_riot\""),
            error: "history/dry_law.toml:10: события dry_law_riot нет в events/",
        },
        Defect {
            path: "events/perestroika/pyramid_victims.toml",
            edit: |text| text.replace("timeline = \"mmm\"", "timeline = \"mmm_crash\""),
            error: "events/perestroika/pyramid_victims.toml:6: узла таймлайна mmm_crash нет в history/ и history/post2004/",
        },
        Defect {
            path: "history/post2004/tech_wave.toml",
            edit: |text| text.replace("weight = 1\nrest_days", "weight = 0\nrest_days"),
            error: "history/post2004/tech_wave.toml:2: вес шаблона должен быть больше нуля",
        },
    ];
    for Defect { path, edit, error } in defects {
        let errors: Vec<String> = Content::from_files(with_file(path, edit))
            .expect_err("набор отвергнут")
            .iter()
            .map(ContentError::to_string)
            .collect();
        assert!(
            errors.iter().any(|reported| reported == error),
            "{path}: ждали «{error}», а пришло {errors:?}"
        );
    }
}

#[test]
fn a_node_and_a_template_may_not_share_a_name() {
    let mut files = Content::read_files(Path::new(HISTORY)).expect("файлы фикстуры читаются");
    let template = files
        .iter()
        .find(|file| file.path == "history/post2004/tech_wave.toml")
        .expect("шаблон есть")
        .text
        .clone();
    files.push(ContentFile {
        path: "history/post2004/mmm.toml".to_owned(),
        text: template,
    });
    let errors: Vec<String> = Content::from_files(files)
        .expect_err("набор отвергнут")
        .iter()
        .map(ContentError::to_string)
        .collect();
    assert!(
        errors.contains(
            &"history/mmm.toml: узел mmm совпадает по имени с шаблоном history/post2004/ — условие timeline их не различит"
                .to_owned()
        ),
        "{errors:?}"
    );
}
