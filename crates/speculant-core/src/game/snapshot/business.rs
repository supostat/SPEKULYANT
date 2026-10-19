//! Дела в снимке: список дел с экранами управляющих, кандидаты в управляющие и дела, которые
//! можно открыть в городе.

use super::super::Game;
use super::super::consequences::signed;
use super::super::russian_count::count;
use crate::business::{Business, Employee, MonthReport};
use crate::content::{Content, HabitId};
use crate::economy::{Currency, Money, MoneyRules};
use crate::time::GameDate;
use crate::view::{
    BusinessKindView, BusinessView, CandidateView, ForecastView, LevelView, ManagerView,
    PolicyView, ReportView, SummaryLineView,
};

const POINTS_SEPARATOR: &str = "·";
const POINTS: [&str; 3] = ["точка", "точки", "точек"];
const SINCE: &str = "с";
const SINCE_YEAR: &str = "года";
const CLOSED_UNTIL: &str = "закрыто до";
const PERCENT_SIGN: char = '%';
const HONESTY: &str = "Честность";
const SKILL: &str = "Хватка";
const LOYALTY: &str = "Лояльность";
const INCOME: &str = "Выручка";
const ROOF: &str = "Крыше";
const TAX: &str = "Налог";
const RENT: &str = "Аренда";
const WAGES: &str = "Зарплата";
const BRIBES: &str = "Откупы";
const STOLEN: &str = "Недостача";
const TOTAL: &str = "Итог";
const PLUS: char = '+';
const MINUS: char = '−';

pub(super) fn businesses(game: &Game, today: GameDate) -> Vec<BusinessView> {
    game.businesses
        .values()
        .map(|business| business_view(game, business, today))
        .collect()
}

/// Кандидаты в управляющие по порядку идентификаторов; из зоны никого не наймёшь.
pub(super) fn candidates(game: &Game, today: GameDate) -> Vec<CandidateView> {
    let content = &game.content;
    if game.zone.is_some() {
        return Vec::new();
    }
    game.candidates(today)
        .into_iter()
        .map(|npc| {
            let template = &content.npcs[npc];
            let traits = template
                .employee
                .as_ref()
                .expect("в кандидатах только люди с чертами сотрудника");
            CandidateView {
                candidate: npc.to_string(),
                name: template.name.clone(),
                role: template.role.clone(),
                traits: trait_levels(content, traits.honesty, traits.skill, traits.loyalty),
                habits: habit_texts(content, &traits.habits),
            }
        })
        .collect()
}

/// Дела, которые сейчас можно открыть в городе игрока; в дороге и в зоне — никаких.
pub(super) fn kinds(game: &Game, today: GameDate) -> Vec<BusinessKindView> {
    let content = &game.content;
    if game.journey.is_some() || game.zone.is_some() {
        return Vec::new();
    }
    let cash = game.player.money().cash();
    content
        .business_kinds
        .iter()
        .filter_map(|(id, kind)| {
            let (_, form) = kind.form_on(&content.business_forms, today)?;
            let cost = content
                .money
                .from_real(kind.point_cost, Currency::Ruble, today);
            let points = game
                .businesses
                .values()
                .find(|business| &business.id().kind == id && business.id().city == game.city)
                .map_or(0, Business::points);
            Some(BusinessKindView {
                kind: id.to_string(),
                title: kind.title.clone(),
                form: form.title.clone(),
                cost: content.money.format_money(cost, today),
                available: cash.covers(cost) && points < kind.max_points,
            })
        })
        .collect()
}

fn business_view(game: &Game, business: &Business, today: GameDate) -> BusinessView {
    let content = &game.content;
    let kind = business.kind(content);
    let rules = &content.business;
    BusinessView {
        business: business.id().to_string(),
        title: format!(
            "{} {POINTS_SEPARATOR} {}",
            kind.title,
            count(business.points(), POINTS)
        ),
        city: content.cities[&business.id().city].name.clone(),
        form: business.form(content, today).title.clone(),
        manager: business.manager().map(|manager| {
            let employee = manager.employee();
            let template = &content.npcs[employee.npc()];
            ManagerView {
                employee: employee.npc().to_string(),
                name: template.name.clone(),
                since: format!("{SINCE} {} {SINCE_YEAR}", manager.since().year()),
                traits: employee_levels(content, employee),
                habits: habit_texts(content, employee.habits()),
            }
        }),
        policies: content
            .policies
            .iter()
            .filter_map(|(kind, policy)| {
                let selected = business.policies().selected(*kind)?;
                Some(PolicyView {
                    policy: kind.id().to_owned(),
                    label: policy.title.clone(),
                    options: policy
                        .options
                        .iter()
                        .map(|option| option.label.clone())
                        .collect(),
                    selected,
                })
            })
            .collect(),
        forecast: business
            .forecast(content, today)
            .map(|forecast| ForecastView {
                revenue: format!("{}{PERCENT_SIGN}", signed(forecast.revenue_percent)),
                revenue_falls: forecast.revenue_percent < 0,
                raid_risk: rules.risk(forecast.raid).name.clone(),
                raid_risk_high: forecast.raid >= rules.alarm_level,
                theft_risk: rules.risk(forecast.theft).name.clone(),
                theft_risk_high: forecast.theft >= rules.alarm_level,
            }),
        closed_until: business
            .closed_until()
            .filter(|until| today < *until)
            .map(|until| format!("{CLOSED_UNTIL} {}", until.numeric())),
        report: business
            .last_report()
            .map(|report| report_view(&content.money, report)),
    }
}

/// Строки отчёта: выручка, аренда, зарплата и итог всегда, крыша, налог, откупы и недостача —
/// когда они были.
fn report_view(rules: &MoneyRules, report: &MonthReport) -> ReportView {
    let date = report.settled_on;
    let line = |label: &str, amount: Money| SummaryLineView {
        label: label.to_owned(),
        value: rules.format_money(amount, date),
    };
    let mut lines = vec![line(INCOME, report.income)];
    for (label, amount) in [(ROOF, report.roof), (TAX, report.tax)] {
        if !amount.is_zero() {
            lines.push(line(label, amount));
        }
    }
    lines.push(line(RENT, report.rent));
    lines.push(line(WAGES, report.wages));
    for (label, amount) in [(BRIBES, report.bribes), (STOLEN, report.stolen)] {
        if !amount.is_zero() {
            lines.push(line(label, amount));
        }
    }
    let profit = report.profit.minor_units();
    let total = rules.format_money(Money::new(Currency::Ruble, profit.abs()), date);
    let sign = if profit < 0 { MINUS } else { PLUS };
    lines.push(SummaryLineView {
        label: TOTAL.to_owned(),
        value: format!("{sign}{total}"),
    });
    ReportView {
        month: date.previous().month_and_year(),
        lines,
    }
}

fn employee_levels(content: &Content, employee: &Employee) -> Vec<LevelView> {
    trait_levels(
        content,
        employee.honesty(),
        employee.skill(),
        employee.loyalty(),
    )
}

/// Черты так, как их рисует экран управляющего: «Честность ▓▓▓░░», «Хватка», «Лояльность».
fn trait_levels(content: &Content, honesty: u8, skill: u8, loyalty: u8) -> Vec<LevelView> {
    [(HONESTY, honesty), (SKILL, skill), (LOYALTY, loyalty)]
        .into_iter()
        .map(|(label, level)| LevelView {
            label: label.to_owned(),
            level,
            scale: content.business.trait_scale,
        })
        .collect()
}

fn habit_texts(content: &Content, habits: &[HabitId]) -> Vec<String> {
    habits
        .iter()
        .map(|habit| content.habits[habit].text.clone())
        .collect()
}
