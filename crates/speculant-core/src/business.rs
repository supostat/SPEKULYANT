//! Своё дело (§7): формы собственности по эпохам, типы дел, точки, управляющие с чертами и
//! привычками, шесть политик, прогноз на месяц, расчёт месяца и происшествия — те, что политика
//! закрывает сама, и те, что выходят за неё и приходят событием.

mod business_id;
mod business_rules;
mod employee;
mod forecast;
mod handling;
mod incident;
mod inspection_response;
mod manager;
mod month_ledger;
mod month_report;
mod monthly_budget;
mod oversight;
mod policies;
mod policy_kind;

pub use business_id::BusinessId;
pub use business_rules::{BusinessRules, RiskLevel};
pub use employee::Employee;
pub use forecast::Forecast;
pub use handling::Handling;
pub use incident::Incident;
pub use inspection_response::InspectionResponse;
pub use manager::Manager;
pub use month_ledger::MonthLedger;
pub use month_report::MonthReport;
pub use monthly_budget::{MonthlyBudget, Operation};
pub use oversight::Oversight;
pub use policies::Policies;
pub use policy_kind::PolicyKind;

use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::content::{BusinessForm, BusinessKind, Content};
use crate::economy::{Currency, Markets, Money, Real};
use crate::events::EventEngine;
use crate::time::GameDate;
use crate::zone::TrusteeConduct;

const PERCENT: f64 = 100.0;
/// Исчезнувшее доверенное лицо уносит всю выручку месяца.
const WHOLE_INCOME: f64 = 1.0;

/// Дело игрока (§7): тип в городе, точки, варианты политик, управляющий, закрытие до даты, учёт
/// месяца и последний отчёт. Без управляющего дело стоит и только платит аренду.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Business {
    id: BusinessId,
    points: u32,
    policies: Policies,
    manager: Option<Manager>,
    closed_until: Option<GameDate>,
    ledger: MonthLedger,
    last_report: Option<MonthReport>,
}

impl Business {
    /// Новое дело в одну точку с политиками по умолчанию и без управляющего.
    pub fn open(id: BusinessId, content: &Content) -> Self {
        let policies = Policies::defaults(&content.business_kinds[&id.kind], content);
        Self {
            id,
            points: 1,
            policies,
            manager: None,
            closed_until: None,
            ledger: MonthLedger::default(),
            last_report: None,
        }
    }

    pub fn id(&self) -> &BusinessId {
        &self.id
    }

    pub fn kind<'c>(&self, content: &'c Content) -> &'c BusinessKind {
        &content.business_kinds[&self.id.kind]
    }

    /// Форма собственности на дату: формы типа без разрывов с первой открытой — это проверяет
    /// загрузка, а дело открывается, только когда одна из них открыта.
    pub fn form<'c>(&self, content: &'c Content, date: GameDate) -> &'c BusinessForm {
        self.kind(content)
            .form_on(&content.business_forms, date)
            .map(|(_, form)| form)
            .expect("у открытого дела всегда есть форма собственности")
    }

    /// Имя дела в сообщениях и событиях: «Ларьки, Москва».
    pub fn name(&self, content: &Content) -> String {
        format!(
            "{}, {}",
            self.kind(content).title,
            content.cities[&self.id.city].name
        )
    }

    pub fn points(&self) -> u32 {
        self.points
    }

    pub fn add_point(&mut self) {
        self.points += 1;
    }

    pub fn policies(&self) -> &Policies {
        &self.policies
    }

    /// Выбирает вариант политики; есть ли они у дела, проверяет вызывающий.
    pub fn set_policy(&mut self, policy: PolicyKind, option: usize) {
        self.policies.set(policy, option);
    }

    pub fn manager(&self) -> Option<&Manager> {
        self.manager.as_ref()
    }

    /// Ставит управляющего; есть ли уже другой, проверяет вызывающий.
    pub fn hire(&mut self, employee: Employee, today: GameDate) {
        assert!(
            self.manager.is_none(),
            "на месте управляющего уже кто-то есть"
        );
        self.manager = Some(Manager::new(employee, today));
    }

    /// Управляющий уходит; дело стоит, пока не наймут другого.
    pub fn dismiss(&mut self) -> Option<Manager> {
        self.manager.take()
    }

    pub fn closed_until(&self) -> Option<GameDate> {
        self.closed_until
    }

    pub fn is_closed_on(&self, date: GameDate) -> bool {
        self.closed_until.is_some_and(|until| date < until)
    }

    /// Закрывает дело с завтрашнего дня на `days` дней; закрытие не сокращает уже идущее.
    pub fn close_for(&mut self, days: u32, today: GameDate) {
        let until = today.after_days(days + 1);
        if self.closed_until.is_none_or(|current| current < until) {
            self.closed_until = Some(until);
        }
    }

    /// Учёт текущего месяца.
    pub fn ledger(&self) -> &MonthLedger {
        &self.ledger
    }

    pub fn last_report(&self) -> Option<&MonthReport> {
        self.last_report.as_ref()
    }

    /// Прогноз на месяц; без управляющего прогноза нет — дело стоит.
    pub fn forecast(&self, content: &Content, date: GameDate) -> Option<Forecast> {
        let manager = self.manager.as_ref()?;
        Some(Forecast::new(
            content,
            self.kind(content),
            self.form(content, date),
            &self.policies,
            manager.employee(),
        ))
    }

    /// Деньги полного месяца с нынешними точками, политиками и управляющим; без управляющего —
    /// как с человеком средней хватки без привычек.
    pub fn budget(
        &self,
        content: &Content,
        markets: &Markets,
        competitors: u32,
        date: GameDate,
    ) -> MonthlyBudget {
        let (revenue_percent, skill) = match (&self.manager, self.forecast(content, date)) {
            (Some(manager), Some(forecast)) => {
                (forecast.revenue_percent, manager.employee().skill())
            }
            _ => (
                self.policies
                    .chosen(content)
                    .map(|(_, option)| option.revenue)
                    .sum(),
                content.business.neutral_trait,
            ),
        };
        let operation = Operation {
            points: self.points,
            revenue_percent,
            skill,
            wage_percent: self.policies.wage_percent(content),
            competitors,
        };
        MonthlyBudget::plan(
            content,
            markets,
            self.kind(content),
            &self.id.city,
            operation,
            date,
        )
    }

    /// Учёт дня: дело есть, при управляющем, работает или закрыто.
    pub fn count_day(&mut self, date: GameDate) {
        self.ledger.days += 1;
        if self.manager.is_some() {
            self.ledger.staffed_days += 1;
            if !self.is_closed_on(date) {
                self.ledger.working_days += 1;
            }
        }
    }

    /// Расчёт прошлого месяца первого числа: выручка за рабочие дни с колебанием, доля крыше и
    /// налог с неё, аренда за все дни, зарплата за дни с управляющим, откупы, недостача — и итог.
    /// При хозяине недостача бывает, если управляющий проворовался по прогнозу; пока хозяин сидит,
    /// её решает поведение доверенного лица. Привычки управляющего могут стоить ему деления
    /// хватки.
    pub fn settle(
        &mut self,
        content: &Content,
        markets: &Markets,
        competitors: u32,
        oversight: Oversight,
        rng: &mut ChaCha8Rng,
        today: GameDate,
    ) -> MonthReport {
        let rules = &content.business;
        let ledger = std::mem::take(&mut self.ledger);
        let month_days = f64::from(today.previous().days_in_month());
        let budget = self.budget(content, markets, competitors, today);
        let noise = 1.0 + (2.0 * EventEngine::fraction(rng) - 1.0) * rules.revenue_noise;
        let income = budget.income * f64::from(ledger.working_days) / month_days * noise;
        let stolen_share = match oversight {
            Oversight::Owner => match self.forecast(content, today) {
                Some(forecast)
                    if EventEngine::roll(rng, rules.risk(forecast.theft).monthly_chance) =>
                {
                    rules.theft_share
                }
                _ => 0.0,
            },
            Oversight::Trustee(TrusteeConduct::Honest) => 0.0,
            Oversight::Trustee(TrusteeConduct::Steals) => content.zone.trustees.steal_share,
            Oversight::Trustee(TrusteeConduct::Vanishes) => WHOLE_INCOME,
        };
        let mut skill_lost = false;
        if let Some(manager) = &mut self.manager {
            let drinking_bouts = manager
                .employee()
                .habits()
                .iter()
                .filter(|habit| EventEngine::roll(rng, content.habits[*habit].skill_loss_chance))
                .count();
            for _ in 0..drinking_bouts {
                manager.employee_mut().lose_skill();
            }
            skill_lost = drinking_bouts > 0;
        }
        let rubles = |kopecks: f64| {
            content.money.from_real(
                Real::from_kopecks(kopecks.round() as i64),
                Currency::Ruble,
                today,
            )
        };
        let roof_share = f64::from(self.policies.roof_percent(content)) / PERCENT;
        let income_rubles = rubles(income);
        let roof = rubles(income * roof_share);
        let tax = rubles(income * self.form(content, today).tax);
        let rent = rubles(budget.rent * f64::from(ledger.days) / month_days);
        let wages = rubles(budget.wages * f64::from(ledger.staffed_days) / month_days);
        let stolen = rubles(income * stolen_share);
        let bribes = content
            .money
            .from_real(ledger.bribes, Currency::Ruble, today);
        let profit = [roof, tax, rent, wages, stolen, bribes]
            .into_iter()
            .fold(income_rubles, Money::minus);
        let report = MonthReport {
            settled_on: today,
            income: income_rubles,
            roof,
            tax,
            rent,
            wages,
            stolen,
            bribes,
            profit,
            skill_lost,
        };
        self.last_report = Some(report);
        report
    }

    /// Происшествия дня: каждое — со своим шансом в месяц, разложенным на дни месяца. Наезд
    /// делится на рэкет и проверку по форме собственности, крыша поднимает ставку, только если она
    /// есть, уход и предательство зависят от лояльности. Без управляющего или в закрытые дни
    /// ничего не случается.
    pub fn incidents_of_the_day(
        &self,
        content: &Content,
        rng: &mut ChaCha8Rng,
        today: GameDate,
    ) -> Vec<Incident> {
        let Some(forecast) = self.forecast(content, today) else {
            return Vec::new();
        };
        if self.is_closed_on(today) {
            return Vec::new();
        }
        let rules = &content.business;
        let raid = rules.risk(forecast.raid).monthly_chance;
        let racket_share = self.form(content, today).racket_share;
        let has_roof = self.policies.roof_percent(content) > 0;
        let month_days = f64::from(today.days_in_month());
        Incident::ALL
            .into_iter()
            .filter(|incident| {
                let monthly = match incident {
                    Incident::Racket => raid * racket_share,
                    Incident::Inspection => raid * (1.0 - racket_share),
                    Incident::RoofRaise if has_roof => rules.roof_raise_chance,
                    Incident::RoofRaise => 0.0,
                    Incident::Departure => {
                        BusinessRules::by_trait(&rules.departure_chances, forecast.loyalty)
                    }
                    Incident::Betrayal => {
                        BusinessRules::by_trait(&rules.betrayal_chances, forecast.loyalty)
                    }
                    Incident::Fire => rules.fire_chance,
                };
                let daily = 1.0 - (1.0 - monthly).powf(1.0 / month_days);
                EventEngine::roll(rng, daily)
            })
            .collect()
    }

    /// Что политика делает с происшествием: крыша разбирается с рэкетом, управляющий откупается
    /// от проверки долей месячной выручки или закрывает дело; остальное выходит за политики.
    pub fn handle(
        &mut self,
        incident: Incident,
        content: &Content,
        markets: &Markets,
        competitors: u32,
        today: GameDate,
    ) -> Handling {
        match incident {
            Incident::Racket if self.policies.roof_percent(content) > 0 => Handling::ByRoof,
            Incident::Inspection => match self.policies.inspection(content) {
                Some(InspectionResponse::Pay) => {
                    let budget = self.budget(content, markets, competitors, today);
                    let bribe = Real::from_kopecks(
                        (budget.income * content.business.bribe_share).round() as i64,
                    );
                    self.ledger.bribes =
                        Real::from_kopecks(self.ledger.bribes.kopecks() + bribe.kopecks());
                    Handling::Bribed(bribe)
                }
                Some(InspectionResponse::Close) => {
                    self.close_for(content.business.closure_days, today);
                    Handling::Closed {
                        until: self.closed_until.expect("дело только что закрыли до даты"),
                    }
                }
                Some(InspectionResponse::Call) | None => Handling::OutOfPolicy,
            },
            Incident::Racket
            | Incident::RoofRaise
            | Incident::Departure
            | Incident::Betrayal
            | Incident::Fire => Handling::OutOfPolicy,
        }
    }
}
