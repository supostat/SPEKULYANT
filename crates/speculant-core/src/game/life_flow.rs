//! Жизнь персонажа (§11, решения 15.4 и 15.5): рождение по биографии и сиду, день персонажа —
//! старение и деньги под матрасом, — покупка жилья, добровольный уход и финал с итогами.

use std::sync::Arc;

use rand_chacha::ChaCha8Rng;
use rand_chacha::rand_core::Rng;

use super::Game;
use crate::command_error::CommandError;
use crate::content::{Biography, BiographyId, Content, EndingId, HousingId};
use crate::economy::{Currency, Money, Real};
use crate::events::EventEngine;
use crate::life::{Character, LifeSummary};
use crate::time::GameDate;
use crate::view::Screen;

const LIFE_STREAM: &str = "life";
const MOVED: &str = "Переехал";
const PAID_EXTRA: &str = "доплатил";
const GOT_BACK: &str = "выручил";
const STASH_LOST: &str = "из-под матраса пропало";
const SOURCE_SEPARATOR: &str = "·";
const PAGE: &str = "с.";
const NAME: &str = "{name}";
const NAME_UPPER: &str = "{NAME}";
const BORN: &str = "{born}";
const ENDED: &str = "{ended}";
const DATE: &str = "{date}";
const AGE: &str = "{age}";
const YEAR: &str = "{year}";
const BUSINESS: &str = "{business}";

/// Персонаж новой жизни: имя из пулов правил, возраст из диапазона биографии и день рождения в
/// году, после которого 1 января 1984 года ему ровно столько лет; здоровье — стартовое правил,
/// жильё — из биографии.
pub(super) fn born_character(
    content: &Content,
    id: &BiographyId,
    biography: &Biography,
    rng: &mut ChaCha8Rng,
) -> Character {
    let names = &content.life.names;
    let mut pick = |pool: &[String]| pool[(rng.next_u64() % pool.len() as u64) as usize].clone();
    let name = format!(
        "{} {} {}",
        pick(&names.surnames),
        pick(&names.first_names),
        pick(&names.patronymics)
    );
    let spread = u64::from(biography.age.to - biography.age.from) + 1;
    let age =
        biography.age.from + u32::try_from(rng.next_u64() % spread).expect("разброс возраста мал");
    let year = GameDate::START.year() - i32::try_from(age).expect("возраст мал") - 1;
    let first = GameDate::from_ymd(year, 1, 2).expect("2 января бывает каждый год");
    let next_year = GameDate::from_ymd(year + 1, 1, 1).expect("1 января бывает каждый год");
    let days =
        u64::try_from(next_year.day_index() - first.day_index()).expect("год после 2 января");
    let born = first.after_days(u32::try_from(rng.next_u64() % days).expect("дней в году мало"));
    Character::new(
        name,
        id.clone(),
        born,
        content.life.start_health,
        biography.housing.clone(),
    )
}

impl Game {
    /// День персонажа; `true`, если жизнь кончилась. В день рождения после порога лет здоровье
    /// убывает, первого числа жильё может лишиться денег под матрасом, на нуле здоровья приходит
    /// финал здоровья.
    pub(super) fn live_character(&mut self, date: GameDate) -> bool {
        let content = Arc::clone(&self.content);
        let rules = &content.life;
        if self.world.character.has_birthday(date)
            && self.world.character.age_on(date) >= rules.aging_from
        {
            self.world
                .character
                .change_health(-rules.aging_per_year, rules.health_max);
        }
        if date.day_of_month() == 1 {
            self.lose_stash(&content, date);
        }
        if self.world.character.health() <= 0 {
            self.end_life(&rules.health_ending, date);
            return true;
        }
        false
    }

    /// Кража или пожар с шансом жилья в месяц уносят долю денег, лежащих дома.
    fn lose_stash(&mut self, content: &Content, date: GameDate) {
        let housing = &content.housing[self.world.character.housing()];
        let struck = EventEngine::roll(self.world.rng.stream(LIFE_STREAM), housing.stash_risk);
        let Some((place, _)) = content.money_places.iter().find(|(_, place)| place.home) else {
            return;
        };
        let balance = self.world.player.money().balance(place);
        if !struck || balance.is_zero() {
            return;
        }
        let lost = Money::new(
            balance.currency(),
            (balance.minor_units() as f64 * content.life.stash_loss_share).round() as i64,
        );
        self.world.player.lose_from(place, lost);
        let shown = content.money.format_money(lost, date);
        self.log(format!("{}: {STASH_LOST} {shown}", housing.title));
    }

    /// Новое жильё за наличные: прежнее уходит в зачёт по доле своей цены; если зачёт больше
    /// цены, разница — игроку.
    pub(super) fn buy_housing(&mut self, housing: &str) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let today = self.today();
        let id = HousingId::new(housing);
        let data = content
            .housing
            .get(&id)
            .ok_or_else(|| CommandError::UnknownHousing {
                housing: housing.to_owned(),
            })?;
        if self.world.character.housing() == &id {
            return Err(CommandError::AlreadyLivingThere {
                title: data.title.clone(),
            });
        }
        if !data.is_on_sale(today) {
            return Err(CommandError::HousingNotOnSale {
                housing: housing.to_owned(),
                title: data.title.clone(),
            });
        }
        let price = content.money.from_real(
            data.price.expect("жильё в продаже с ценой"),
            Currency::Ruble,
            today,
        );
        let resale = self.resale_of_home(today);
        let message = if price.covers(resale) {
            let due = price.minus(resale);
            let cash = self.world.player.money().cash();
            if !cash.covers(due) {
                return Err(CommandError::NotEnoughCash {
                    needed: content.money.format_money(due, today),
                    cash: content.money.format_money(cash, today),
                });
            }
            self.world.player.pay_cash(due);
            format!(
                "{MOVED}: {}, {PAID_EXTRA} {}",
                data.title,
                content.money.format_money(due, today)
            )
        } else {
            let refund = resale.minus(price);
            self.world.player.receive_cash(refund);
            format!(
                "{MOVED}: {}, {GOT_BACK} {}",
                data.title,
                content.money.format_money(refund, today)
            )
        };
        self.world.character.move_to(id);
        self.log(message);
        Ok(())
    }

    /// Во что идёт в зачёт нынешнее жильё: доля его цены; жильё без цены — ни во что.
    pub(super) fn resale_of_home(&self, today: GameDate) -> Money {
        let content = &self.content;
        content.housing[self.world.character.housing()]
            .price
            .map_or(Money::zero(Currency::Ruble), |price| {
                let share = (price.kopecks() as f64 * content.life.resale_share).round() as i64;
                content
                    .money
                    .from_real(Real::from_kopecks(share), Currency::Ruble, today)
            })
    }

    /// Добровольный уход: эмиграция или покой, когда он открыт.
    pub(super) fn retire(&mut self, ending: &str) -> Result<(), CommandError> {
        let today = self.today();
        let id = EndingId::new(ending);
        let data = self
            .content
            .endings
            .get(&id)
            .ok_or_else(|| CommandError::UnknownEnding {
                ending: ending.to_owned(),
            })?;
        if !data.is_open_on(today) {
            return Err(CommandError::EndingUnavailable {
                ending: ending.to_owned(),
                title: data.title.clone(),
            });
        }
        self.end_life(&id, today);
        Ok(())
    }

    /// Финал: итоги снимаются в этот день, открытия, знания и место в рекордах уходят в
    /// метапрогрессию — и на диск, если он есть, — время останавливается. Если финал пришёл
    /// последствием события, экран итогов откроется, когда игрок прочтёт исход.
    pub(super) fn end_life(&mut self, ending: &EndingId, date: GameDate) {
        let summary = self.summarize(ending, date);
        self.remember_life(&summary);
        self.world.summary = Some(summary);
        self.store_after_life();
        self.world.clock.stop_fast_forward();
        if self.world.events.pending().is_none() {
            self.world.screen = Screen::Summary;
        }
    }

    /// Капитал в рублях 1984 года: деньги, товар по средней цене города, точки дел по цене
    /// открытия и жильё по цене.
    pub(super) fn capital(&self, date: GameDate) -> Real {
        let content = &self.content;
        let money = self.world.player.money().real_total(&content.money, date);
        let goods: f64 = self
            .world
            .player
            .pocket()
            .goods()
            .filter_map(|(good, quantity)| {
                self.world
                    .markets
                    .normal_kopecks(content, &self.world.city, good, date)
                    .map(|price| price * f64::from(quantity))
            })
            .sum();
        let businesses: i64 = self
            .world
            .businesses
            .values()
            .map(|business| {
                business.kind(content).point_cost.kopecks() * i64::from(business.points())
            })
            .sum();
        let housing = content.housing[self.world.character.housing()]
            .price
            .map_or(0, Real::kopecks);
        Real::from_kopecks(money.kopecks() + goods.round() as i64 + businesses + housing)
    }

    /// Итоги в духе эпохи: заметка той газеты или того суда, что пишет о финале в эту эпоху, на
    /// случайной странице; текст — первая фраза биографии, первое дело и последняя фраза финала.
    fn summarize(&mut self, ending: &EndingId, date: GameDate) -> LifeSummary {
        let content = Arc::clone(&self.content);
        let data = &content.endings[ending];
        let epoch = &content.time.epochs.epoch_on(date).id;
        let source = data
            .sources
            .iter()
            .find(|(id, _)| id.as_str() == epoch)
            .map(|(_, source)| source)
            .expect("у финала заметка для каждой эпохи — это проверяет загрузка");
        let pages = u64::from(content.life.newspaper_pages);
        let page = 1 + self.world.rng.stream(LIFE_STREAM).next_u64() % pages;
        let character = &self.world.character;
        let biography = &content.biographies[character.biography()];
        let fill = |text: &str| {
            text.replace(NAME_UPPER, &character.name().to_uppercase())
                .replace(NAME, character.name())
                .replace(BORN, &character.born().year().to_string())
                .replace(ENDED, &date.year().to_string())
                .replace(DATE, &date.long())
                .replace(AGE, &character.age_on(date).to_string())
        };
        let mut text = vec![biography.obituary.trim().to_owned()];
        if let Some((opened, kind)) = self.world.chronicle.first_business() {
            text.push(
                content
                    .life
                    .business_sentence
                    .replace(YEAR, &opened.year().to_string())
                    .replace(BUSINESS, &content.business_kinds[kind].title),
            );
        }
        text.push(fill(data.text.trim()));
        let flags = self.world.events.flags();
        let discoveries = content
            .biographies
            .iter()
            .filter(|(id, biography)| {
                !biography.starts_unlocked
                    && !self.meta.unlocked().contains(*id)
                    && biography
                        .unlocked_by_flag
                        .as_ref()
                        .is_some_and(|flag| flags.contains(flag))
            })
            .map(|(id, _)| id.clone())
            .collect();
        LifeSummary {
            ending: ending.clone(),
            ended: date,
            source: format!(
                "{source} {SOURCE_SEPARATOR} {} {SOURCE_SEPARATOR} {PAGE} {page}",
                date.long()
            ),
            headline: fill(&data.headline),
            text: text.join(" "),
            capital: self.capital(date),
            best_deal: self.world.chronicle.best_deal().cloned(),
            worst_deal: self.world.chronicle.worst_deal().cloned(),
            businesses: self
                .world
                .businesses
                .values()
                .map(|business| business.name(&content))
                .collect(),
            discoveries,
        }
    }
}
