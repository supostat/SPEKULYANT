//! Зона (§12, решение 15.8): приговор, день и неделя отсидки, промзона, передачи, услуги, УДО,
//! взятка, перемены от событий зоны и выход на волю со связями «оттуда».

use std::sync::Arc;

use super::Game;
use super::consequences::signed;
use super::russian_count::{packs, term};
use crate::command_error::CommandError;
use crate::content::{ZoneChange, ZoneServiceId};
use crate::economy::{Currency, Money};
use crate::events::EventEngine;
use crate::time::GameDate;
use crate::view::Screen;
use crate::zone::{Release, Sentence, Zone};

const ZONE_STREAM: &str = "zone";
const SENTENCED: &str = "Осуждён";
const ADDED_TO_TERM: &str = "К сроку добавили";
const RELEASED: &str = "Вышел на волю";
const TIES: &str = "Связи «оттуда»";
const AUTHORITY: &str = "Авторитет";
const CIGARETTES: &str = "Сигареты";
const TEA: &str = "Чай";
const RELEASE_BY_TERM: &str = "Выход по сроку —";
const PAROLE_REFUSED: &str = "УДО: отказ, следующая просьба — с";
const PARCEL: &str = "Передача с воли";
const BRIBE: &str = "Взятка за выход";
const FOR: &str = "за";

impl Game {
    /// Приговор на `months` месяцев. Срок длиннее порога правил кончает жизнь финалом зоны; в
    /// зоне новый срок прибавляется к нынешнему. Иначе игрок снят с дороги и сидит с этого дня, а
    /// неделя зоны идёт за время одного дня на воле. Если приговор пришёл исходом события, экран
    /// зоны откроется, когда игрок его прочтёт.
    pub(super) fn imprison(&mut self, months: u32, today: GameDate) {
        let content = Arc::clone(&self.content);
        let rules = &content.zone;
        if let Some(zone) = &mut self.zone {
            zone.change_term(i32::try_from(months).expect("срок в месяцах мал"));
            let until = zone.sentence().until();
            self.log(format!(
                "{ADDED_TO_TERM} {}. {RELEASE_BY_TERM} {}",
                term(months),
                until.numeric()
            ));
            self.end_life_if_term_too_long(today);
            return;
        }
        if months > rules.longest_sentence_months {
            self.end_life(&rules.life_ending, today);
            return;
        }
        self.journey = None;
        self.zone = Some(Zone::new(Sentence::new(today, months), rules));
        self.clock.stop_fast_forward();
        self.clock.set_zone_pace(rules.week_days);
        self.log(format!(
            "{SENTENCED}: {}, {}",
            term(months),
            content.camp_on(today).title
        ));
        if self.events.pending().is_none() {
            self.screen = Screen::Zone;
        }
    }

    /// Срок, выросший за порог правил, кончает жизнь финалом зоны.
    fn end_life_if_term_too_long(&mut self, today: GameDate) {
        let content = Arc::clone(&self.content);
        let rules = &content.zone;
        let months = self
            .zone
            .as_ref()
            .expect("срок растёт только в зоне")
            .sentence()
            .months();
        if months > rules.longest_sentence_months {
            self.end_life(&rules.life_ending, today);
        }
    }

    /// День зоны; `true`, если он кончился выходом на волю, событием или смертью. Отсидел срок —
    /// выходит; продолжения цепочек приходят в свой день; в конце недели промзона платит,
    /// безделье растит авторитет, зона подтачивает здоровье и может случиться событие зоны.
    pub(super) fn live_zone_day(&mut self, date: GameDate) -> bool {
        let sentence = *self
            .zone
            .as_ref()
            .expect("день зоны — только в зоне")
            .sentence();
        if sentence.is_served_on(date) {
            self.release(Release::Term);
            return true;
        }
        if self.start_due_follow_up(date) {
            return true;
        }
        if !sentence.is_week_end(date, self.content.zone.week_days) {
            return false;
        }
        self.live_zone_week(date)
    }

    fn live_zone_week(&mut self, date: GameDate) -> bool {
        let content = Arc::clone(&self.content);
        let rules = &content.zone;
        let zone = self.zone.as_mut().expect("неделя зоны — только в зоне");
        let before = zone.authority();
        zone.live_week(rules, content.camp_on(date), self.rng.stream(ZONE_STREAM));
        let authority = zone.authority();
        if authority != before {
            self.log(format!("{AUTHORITY}: {}", rules.authority_title(authority)));
        }
        if EventEngine::roll(self.rng.stream(ZONE_STREAM), rules.health_loss_chance) {
            self.character
                .change_health(-rules.health_loss, content.life.health_max);
            if self.character.health() <= 0 {
                self.end_life(&content.life.health_ending, date);
                return true;
            }
        }
        self.draw_event(date, rules.event_chance)
    }

    /// Выход на волю: время снова идёт днями, ставится флаг выхода, а авторитет оборачивается
    /// связями «оттуда» — репутацией фракции по таблице правил. Сигареты и чай остаются в зоне.
    /// Если выход пришёл исходом события, рынок откроется, когда игрок его прочтёт.
    pub(super) fn release(&mut self, release: Release) {
        let content = Arc::clone(&self.content);
        let rules = &content.zone;
        let zone = self.zone.take().expect("выходят только из зоны");
        self.clock.stop_fast_forward();
        self.clock.set_freedom_pace();
        self.clock.drop_carried_time();
        self.events.set_flag(rules.release_flag.clone());
        self.log(format!("{RELEASED} {}", release.title()));
        let ties = rules.ties[usize::from(zone.authority()) - 1];
        if ties != 0 {
            self.reputation
                .change(&rules.ties_faction, ties, content.reputation);
            self.log(format!(
                "{TIES}: {} {}",
                content.factions[&rules.ties_faction].title,
                signed(ties)
            ));
        }
        if self.events.pending().is_none() {
            self.screen = Screen::Market;
        }
    }

    /// Перемена от события зоны: сигареты и чай, авторитет, месяцы срока и амнистия.
    pub(super) fn change_zone(
        &mut self,
        change: &ZoneChange,
        today: GameDate,
        effects: &mut Vec<String>,
    ) {
        let content = Arc::clone(&self.content);
        let rules = &content.zone;
        let zone = self
            .zone
            .as_mut()
            .expect("событие зоны с переменой в зоне приходит только в зоне");
        zone.pay(change.price());
        zone.receive(change.gain());
        if change.cigarettes != 0 {
            effects.push(format!("{CIGARETTES} {}", signed(change.cigarettes)));
        }
        if change.tea != 0 {
            effects.push(format!("{TEA} {}", signed(change.tea)));
        }
        if change.authority != 0 {
            zone.change_authority(change.authority, rules);
            effects.push(format!(
                "{AUTHORITY}: {}",
                rules.authority_title(zone.authority())
            ));
        }
        if change.term_months != 0 {
            zone.change_term(change.term_months);
            effects.push(format!(
                "{RELEASE_BY_TERM} {}",
                zone.sentence().until().numeric()
            ));
            self.end_life_if_term_too_long(today);
        }
        if change.amnesty && self.summary.is_none() {
            self.release(Release::Amnesty);
            effects.push(format!("{RELEASED} {}", Release::Amnesty.title()));
        }
    }

    /// На промзону или нет: работа платит сигаретами и пишет трудовую характеристику для УДО, а
    /// безделье растит авторитет.
    pub(super) fn set_work(&mut self, work: bool) -> Result<(), CommandError> {
        self.zone
            .as_mut()
            .ok_or(CommandError::NotInZone)?
            .set_working(work);
        Ok(())
    }

    /// Услуга за сигареты и чай: здоровье — в пределах шкалы, авторитет — в пределах уровней.
    pub(super) fn use_service(&mut self, service: &str) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let zone = self.zone.as_mut().ok_or(CommandError::NotInZone)?;
        let data = content
            .zone_services
            .get(&ZoneServiceId::new(service))
            .ok_or_else(|| CommandError::UnknownService {
                service: service.to_owned(),
            })?;
        if !zone.stash().covers(data.price) {
            return Err(CommandError::StashTooSmall {
                title: data.title.clone(),
                needed: packs(data.price),
                stash: packs(zone.stash()),
            });
        }
        zone.pay(data.price);
        zone.change_authority(data.authority, &content.zone);
        self.character
            .change_health(data.health, content.life.health_max);
        self.log(format!("{} {FOR} {}", data.title, packs(data.price)));
        Ok(())
    }

    /// Передача с воли за наличные: сигареты и чай приходят сразу, следующую зона эпохи разрешит
    /// через свой срок.
    pub(super) fn order_parcel(&mut self) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let rules = &content.zone;
        let today = self.today();
        let zone = self.zone.as_ref().ok_or(CommandError::NotInZone)?;
        if today < zone.next_parcel() {
            return Err(CommandError::ParcelTooSoon {
                from: zone.next_parcel().numeric(),
            });
        }
        let price = content
            .money
            .from_real(rules.parcel.price, Currency::Ruble, today);
        self.pay_from_cash(price, today)?;
        let next = rules.weeks_after(today, content.camp_on(today).parcel_weeks);
        self.zone
            .as_mut()
            .expect("в зоне — проверено выше")
            .receive_parcel(rules.parcel.contents, next);
        self.log(format!(
            "{PARCEL}: {} {FOR} {}",
            packs(rules.parcel.contents),
            content.money.format_money(price, today)
        ));
        Ok(())
    }

    /// Просьба об УДО, когда отсижена доля срока зоны эпохи: комиссия даёт его с шансом по
    /// трудовой характеристике и авторитету, а после отказа следующая просьба — через срок правил.
    pub(super) fn apply_for_parole(&mut self) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let rules = &content.zone;
        let today = self.today();
        let camp = content.camp_on(today);
        let zone = self.zone.as_ref().ok_or(CommandError::NotInZone)?;
        let from = zone.parole_from(camp);
        if today < from {
            return Err(CommandError::ParoleTooEarly {
                from: from.numeric(),
            });
        }
        let chance = zone.parole_chance(rules, camp);
        if EventEngine::roll(self.rng.stream(ZONE_STREAM), chance) {
            self.release(Release::Parole);
            return Ok(());
        }
        let next = rules.weeks_after(today, rules.parole_retry_weeks);
        self.zone
            .as_mut()
            .expect("в зоне — проверено выше")
            .refuse_parole(next);
        self.log(format!("{PAROLE_REFUSED} {}", next.numeric()));
        Ok(())
    }

    /// Выход за взятку наличными — если зона эпохи его продаёт.
    pub(super) fn bribe_for_release(&mut self) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let today = self.today();
        if self.zone.is_none() {
            return Err(CommandError::NotInZone);
        }
        let camp = content.camp_on(today);
        let bribe = camp.bribe.ok_or_else(|| CommandError::NoBribe {
            camp: camp.title.clone(),
        })?;
        let price = content.money.from_real(bribe, Currency::Ruble, today);
        self.pay_from_cash(price, today)?;
        self.log(format!(
            "{BRIBE}: {}",
            content.money.format_money(price, today)
        ));
        self.release(Release::Bribe);
        Ok(())
    }

    /// Платит наличными на воле: передачу и взятку оплачивают из них.
    fn pay_from_cash(&mut self, price: Money, today: GameDate) -> Result<(), CommandError> {
        let cash = self.player.money().cash();
        if !cash.covers(price) {
            return Err(CommandError::NotEnoughCash {
                needed: self.content.money.format_money(price, today),
                cash: self.content.money.format_money(cash, today),
            });
        }
        self.player.pay_cash(price);
        Ok(())
    }
}
