//! Дела игрока (§7): открытие точек, найм и увольнение управляющих, варианты политик и день дел —
//! расчёт прошлого месяца первого числа, учёт дня и происшествия, которые политика закрывает сама
//! или отдаёт хозяину событием.

use std::sync::Arc;

use super::Game;
use crate::business::{
    Business, BusinessId, Employee, Handling, MonthReport, Oversight, PolicyKind,
};
use crate::command_error::CommandError;
use crate::content::{BusinessKindId, NpcId};
use crate::economy::{Currency, Money};
use crate::reputation::{Contact, ContactRefusal};
use crate::time::GameDate;
use crate::zone::TrusteeConduct;

const BUSINESS_STREAM: &str = "business";
const OPENED: &str = "Открыл";
const NEW_POINT: &str = "ещё одна точка";
const MANAGER: &str = "управляющий";
const HIRED: &str = "Нанят";
const FIRED: &str = "Уволен";
const DISMISSED: &str = "уволен";
const PROFIT: &str = "прибыль";
const LOSS: &str = "убыток";
const BANKRUPT: &str = "покрыть нечем — дело закрыто";
const DRINKS: &str = "пьёт: хватка";
const OUT_OF: &str = "из";
const RAID_BY_ROOF: &str = "наезд — крыша разобралась";
const INSPECTION_BRIBED: &str = "проверка — откупились за";
const INSPECTION_CLOSED: &str = "проверка — закрылись до";
const VANISHED: &str = "исчез с выручкой";

impl Game {
    /// Точка дела в городе игрока: дело открывается в форме собственности дня, вторая и следующие
    /// точки того же типа расширяют его до предела типа. Платят наличными по цене точки.
    pub(super) fn open_business(&mut self, kind: &str) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let today = self.today();
        let kind_id = BusinessKindId::new(kind);
        let data = content.business_kinds.get(&kind_id).ok_or_else(|| {
            CommandError::UnknownBusinessKind {
                kind: kind.to_owned(),
            }
        })?;
        if data.form_on(&content.business_forms, today).is_none() {
            return Err(CommandError::BusinessNotYet {
                kind: kind.to_owned(),
                title: data.title.clone(),
            });
        }
        let id = BusinessId::new(kind_id, self.city.clone());
        let points = self.businesses.get(&id).map_or(0, Business::points);
        if points >= data.max_points {
            return Err(CommandError::MaxPoints {
                kind: kind.to_owned(),
                title: data.title.clone(),
                max: data.max_points,
            });
        }
        let cost = content
            .money
            .from_real(data.point_cost, Currency::Ruble, today);
        let cash = self.player.money().cash();
        if !cash.covers(cost) {
            return Err(CommandError::NotEnoughCash {
                needed: content.money.format_money(cost, today),
                cash: content.money.format_money(cash, today),
            });
        }
        self.player.pay_cash(cost);
        let price = content.money.format_money(cost, today);
        let message = match self.businesses.get_mut(&id) {
            Some(business) => {
                business.add_point();
                format!("{}: {NEW_POINT} за {price}", business.name(&content))
            }
            None => {
                self.chronicle.opened_business(&id.kind, today);
                let business = Business::open(id.clone(), &content);
                let message = format!("{OPENED}: {} за {price}", business.name(&content));
                self.businesses.insert(id, business);
                message
            }
        };
        self.log(message);
        Ok(())
    }

    /// Вариант политики дела — номер от нуля.
    pub(super) fn set_policy(
        &mut self,
        business: &str,
        policy: &str,
        option: usize,
    ) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let id = self.known_business(business)?;
        let owned = &self.businesses[&id];
        let kind = PolicyKind::from_id(policy)
            .filter(|kind| owned.kind(&content).has_policy(*kind))
            .ok_or_else(|| CommandError::UnknownPolicy {
                business: business.to_owned(),
                policy: policy.to_owned(),
            })?;
        let count = content.policies[&kind].options.len();
        if option >= count {
            return Err(CommandError::NoSuchPolicyOption {
                policy: policy.to_owned(),
                option,
                count,
            });
        }
        self.businesses
            .get_mut(&id)
            .expect("дело нашлось выше")
            .set_policy(kind, option);
        Ok(())
    }

    /// Нанимает управляющего из кандидатов: место должно быть свободно, человек — не занят другим
    /// делом игрока, на связи и не обижен. Нанятый становится связью.
    pub(super) fn hire(&mut self, business: &str, candidate: &str) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let today = self.today();
        let id = self.known_business(business)?;
        let npc = NpcId::new(candidate);
        let (template, traits) = content
            .npcs
            .get(&npc)
            .and_then(|template| Some((template, template.employee.as_ref()?)))
            .ok_or_else(|| CommandError::UnknownCandidate {
                candidate: candidate.to_owned(),
            })?;
        let name = self.businesses[&id].name(&content);
        if let Some(manager) = self.businesses[&id].manager() {
            return Err(CommandError::ManagerInPlace {
                business: name,
                name: content.npcs[manager.employee().npc()].name.clone(),
            });
        }
        if let Some(employer) = self.employer_of(&npc) {
            return Err(CommandError::CandidateBusy {
                candidate: candidate.to_owned(),
                name: template.name.clone(),
                business: employer.name(&content),
            });
        }
        if let Some(refusal) = self.candidate_refusal(&npc, today) {
            return Err(CommandError::CandidateRefuses {
                candidate: candidate.to_owned(),
                name: template.name.clone(),
                reason: refusal.to_string(),
            });
        }
        self.businesses
            .get_mut(&id)
            .expect("дело нашлось выше")
            .hire(Employee::from_template(npc.clone(), traits), today);
        self.contacts
            .entry(npc.clone())
            .or_insert_with(|| Contact::new(npc))
            .record(today, &format!("{HIRED}: {name}"));
        self.log(format!("{name}: {MANAGER} — {}", template.name));
        Ok(())
    }

    /// Увольняет управляющего: дело стоит, пока не наймут другого, а уволенный обижается.
    pub(super) fn fire(&mut self, business: &str, employee: &str) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let today = self.today();
        let id = self.known_business(business)?;
        let owned = self.businesses.get_mut(&id).expect("дело нашлось выше");
        if owned
            .manager()
            .is_none_or(|manager| manager.employee().npc().as_str() != employee)
        {
            return Err(CommandError::NotManager {
                business: business.to_owned(),
                employee: employee.to_owned(),
            });
        }
        let name = owned.name(&content);
        let manager = owned.dismiss().expect("управляющий есть — проверено выше");
        let npc = manager.employee().npc().clone();
        self.contacts
            .entry(npc.clone())
            .or_insert_with(|| Contact::new(npc.clone()))
            .change_relation(
                content.business.fired_relation,
                content.reputation,
                today,
                FIRED,
            );
        self.log(format!("{name}: {} {DISMISSED}", content.npcs[&npc].name));
        Ok(())
    }

    /// Кандидаты в управляющие: люди с чертами сотрудника, не занятые делами игрока, на связи и
    /// не обиженные.
    pub(super) fn candidates(&self, today: GameDate) -> Vec<&NpcId> {
        self.content
            .npcs
            .iter()
            .filter(|(npc, template)| {
                template.employee.is_some()
                    && self.employer_of(npc).is_none()
                    && self.candidate_refusal(npc, today).is_none()
            })
            .map(|(npc, _)| npc)
            .collect()
    }

    /// День дел; `true`, если происшествие, вышедшее за политики, началось событием. Первого
    /// числа каждое дело считает прошлый месяц, затем учитывает день и разыгрывает происшествия:
    /// закрытые политикой пишутся в ленту, остальные ждут события — первое готовое начинается,
    /// прочие в этот день проходят без последствий.
    pub(super) fn live_businesses(&mut self, date: GameDate) -> bool {
        let content = Arc::clone(&self.content);
        let ids: Vec<BusinessId> = self.businesses.keys().cloned().collect();
        let mut messages = Vec::new();
        let mut out_of_policy = Vec::new();
        for id in ids {
            let competitors = self.rivals.business_points(&id.kind, &id.city);
            if date.day_of_month() == 1 && self.businesses[&id].ledger().days > 0 {
                let oversight = self.oversight(&id, date);
                let report = self
                    .businesses
                    .get_mut(&id)
                    .expect("дело из списка ключей")
                    .settle(
                        &content,
                        &self.markets,
                        competitors,
                        oversight,
                        self.rng.stream(BUSINESS_STREAM),
                        date,
                    );
                if oversight == Oversight::Trustee(TrusteeConduct::Vanishes) {
                    self.trustee_vanishes(&id, date, &mut messages);
                }
                if !self.pay_month(&id, report, &mut messages) {
                    continue;
                }
            }
            let business = self.businesses.get_mut(&id).expect("дело не закрыто");
            business.count_day(date);
            let name = business.name(&content);
            let incidents =
                business.incidents_of_the_day(&content, self.rng.stream(BUSINESS_STREAM), date);
            for incident in incidents {
                match business.handle(incident, &content, &self.markets, competitors, date) {
                    Handling::ByRoof => messages.push(format!("{name}: {RAID_BY_ROOF}")),
                    Handling::Bribed(bribe) => {
                        let bribe = content.money.from_real(bribe, Currency::Ruble, date);
                        messages.push(format!(
                            "{name}: {INSPECTION_BRIBED} {}",
                            content.money.format_money(bribe, date)
                        ));
                    }
                    Handling::Closed { until } => {
                        messages.push(format!("{name}: {INSPECTION_CLOSED} {}", until.numeric()));
                    }
                    Handling::OutOfPolicy => out_of_policy.push((id.clone(), incident)),
                }
            }
        }
        for message in messages {
            self.log(message);
        }
        out_of_policy
            .into_iter()
            .any(|(id, incident)| self.start_incident_event(&id, incident, date))
    }

    /// Кто смотрел за делом месяц: на воле — хозяин; пока хозяин сидит, управляющий ведёт дело по
    /// своей лояльности и честности — честно, воруя или исчезнув с выручкой (§12).
    fn oversight(&mut self, id: &BusinessId, date: GameDate) -> Oversight {
        let business = &self.businesses[id];
        let (Some(_), Some(manager), Some(forecast)) = (
            &self.zone,
            business.manager(),
            business.forecast(&self.content, date),
        ) else {
            return Oversight::Owner;
        };
        let honesty = manager.employee().honesty();
        Oversight::Trustee(TrusteeConduct::draw(
            &self.content.zone.trustees,
            forecast.loyalty,
            honesty,
            self.rng.stream(BUSINESS_STREAM),
        ))
    }

    /// Доверенное лицо исчезло с выручкой месяца: дело остаётся без управляющего, а его самого
    /// больше не найти.
    fn trustee_vanishes(&mut self, id: &BusinessId, date: GameDate, messages: &mut Vec<String>) {
        let content = Arc::clone(&self.content);
        let business = self.businesses.get_mut(id).expect("дело из списка ключей");
        let name = business.name(&content);
        let manager = business
            .dismiss()
            .expect("исчезает только управляющий, который был");
        let npc = manager.employee().npc().clone();
        messages.push(format!("{name}: {} {VANISHED}", content.npcs[&npc].name));
        self.contacts
            .entry(npc.clone())
            .or_insert_with(|| Contact::new(npc))
            .lose(date, &format!("{name}: {VANISHED}"));
    }

    /// Итог месяца хозяину: прибыль — в наличные, убыток — из наличных; убыток, который не
    /// покрыть, забирает все наличные и закрывает дело. `false`, если дело закрылось.
    fn pay_month(
        &mut self,
        id: &BusinessId,
        report: MonthReport,
        messages: &mut Vec<String>,
    ) -> bool {
        let content = Arc::clone(&self.content);
        let today = report.settled_on;
        let business = &self.businesses[id];
        let name = business.name(&content);
        let month = today.previous().month_and_year();
        if report.skill_lost
            && let Some(manager) = business.manager()
        {
            messages.push(format!(
                "{} {DRINKS} {} {OUT_OF} {}",
                content.npcs[manager.employee().npc()].name,
                manager.employee().skill(),
                content.business.trait_scale
            ));
        }
        let profit = report.profit.minor_units();
        let amount = Money::new(Currency::Ruble, profit.abs());
        let shown = content.money.format_money(amount, today);
        if profit >= 0 {
            self.player.receive_cash(amount);
            messages.push(format!("{name}: {month} — {PROFIT} {shown}"));
            return true;
        }
        let cash = self.player.money().cash();
        if cash.covers(amount) {
            self.player.pay_cash(amount);
            messages.push(format!("{name}: {month} — {LOSS} {shown}"));
            return true;
        }
        self.player.pay_cash(cash);
        self.businesses.remove(id);
        messages.push(format!("{name}: {month} — {LOSS} {shown}, {BANKRUPT}"));
        false
    }

    fn known_business(&self, business: &str) -> Result<BusinessId, CommandError> {
        BusinessId::parse(business)
            .filter(|id| self.businesses.contains_key(id))
            .ok_or_else(|| CommandError::UnknownBusiness {
                business: business.to_owned(),
            })
    }

    /// Дело игрока, которым человек сейчас управляет.
    fn employer_of(&self, npc: &NpcId) -> Option<&Business> {
        self.businesses.values().find(|business| {
            business
                .manager()
                .is_some_and(|manager| manager.employee().npc() == npc)
        })
    }

    /// Почему человек не пойдёт в управляющие: потерян, в отъезде или обижен. Долг услуг найму
    /// не мешает — это работа, а не одолжение.
    fn candidate_refusal(&self, npc: &NpcId, today: GameDate) -> Option<ContactRefusal> {
        self.contacts
            .get(npc)
            .and_then(|contact| contact.refusal(self.content.reputation, today))
            .filter(|refusal| *refusal != ContactRefusal::OwedTooMuch)
    }
}
