//! Жизнь бота на настоящем ядре: день за днём на первой скорости, неделя за неделей в зоне — до
//! финала или до первого дня бесконечной эпохи; в первый день каждой эпохи бот записывает капитал.

use std::sync::Arc;

use speculant_core::rng::WorldRng;
use speculant_core::time::GameDate;
use speculant_core::{Command, Content, Game, LifeSettings};

use crate::bot::{Bot, Curiosity};
use crate::{LifeOutcome, LifePlan};

/// Поток случая бота: он не сдвигает потоков мира с тем же сидом.
const BOT_STREAM: &str = "bot";
/// Поток любопытства — свой, чтобы без любопытства жизнь шла так же, как до него.
const CURIOSITY_STREAM: &str = "curiosity";
/// На первой скорости за длительность дня проходит ровно день, в зоне — её неделя.
const SPEED: u8 = 1;

/// Первый день последней эпохи календаря: у неё нет конца, и бот проживает историю до неё.
pub fn horizon(content: &Content) -> GameDate {
    content
        .time
        .epochs
        .epochs()
        .last()
        .expect("в календаре есть эпохи — это проверяет загрузка")
        .starts
}

pub fn play_life(content: &Arc<Content>, plan: &LifePlan) -> LifeOutcome {
    let settings = LifeSettings {
        biography: plan.biography.to_string(),
        ironman: true,
        seed: plan.seed,
    };
    let mut game = Game::new_life(&settings, Arc::clone(content))
        .expect("бот берёт биографию, открытую с начала");
    let mut bot = plan.strategy.bot();
    let curiosity = Curiosity {
        share: plan.curiosity,
    };
    let mut dice = WorldRng::new(plan.seed);
    let day = content.time.clock.day_milliseconds_at(SPEED);
    let epoch_ends: Vec<GameDate> = content
        .time
        .epochs
        .epochs()
        .iter()
        .skip(1)
        .map(|epoch| epoch.starts)
        .collect();
    let mut epoch_capitals = Vec::new();
    let mut month_lived = None;
    while game.summary().is_none() && epoch_capitals.len() < epoch_ends.len() {
        if game.today() >= epoch_ends[epoch_capitals.len()] {
            epoch_capitals.push(game.capital());
            continue;
        }
        if game.events().pending().is_some() {
            settle_event(&mut game, &bot, curiosity, &mut dice);
            continue;
        }
        if game.zone().is_some() {
            bot.serve_week(&mut game);
        } else if game.journey().is_none() {
            let month = (game.today().year(), game.today().month());
            if month_lived != Some(month) {
                month_lived = Some(month);
                bot.live_month(&mut game);
            }
            bot.live_day(&mut game, dice.stream(BOT_STREAM));
        }
        game.advance(day);
    }
    LifeOutcome {
        strategy: plan.strategy,
        biography: plan.biography.clone(),
        seed: plan.seed,
        ending: game.summary().map(|summary| summary.ending.clone()),
        last_day: game.today(),
        capital: game
            .summary()
            .map_or_else(|| game.capital(), |summary| summary.capital),
        epoch_capitals,
        events: game
            .journal()
            .iter()
            .map(|entry| (entry.date, entry.event.clone()))
            .collect(),
    }
}

/// Бот выбирает — из любопытства или своей манерой, — если выбор ещё не сделан, и читает исход.
fn settle_event(game: &mut Game, bot: &Bot, curiosity: Curiosity, dice: &mut WorldRng) {
    let event = game
        .view()
        .pending_event
        .expect("событие ждёт ответа — проверено выше");
    if event.outcome.is_none() {
        let choice = curiosity
            .pick(game, &event, dice.stream(CURIOSITY_STREAM))
            .unwrap_or_else(|| bot.choose(game, &event, dice.stream(BOT_STREAM)));
        game.apply(Command::Choose { choice })
            .expect("бот выбирает доступный выбор");
    }
    game.apply(Command::Continue).expect("исход прочитан");
}
