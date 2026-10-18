//! Поездки в жизни (§4): выезд с оплатой проезда и, где нужно, услугой знакомого, дорожные
//! события маршрута и прибытие.

use std::sync::Arc;

use super::Game;
use crate::command_error::CommandError;
use crate::content::{CityId, Route};
use crate::economy::Currency;
use crate::events::EventEngine;
use crate::time::GameDate;
use crate::travel::{Access, Journey};

const EVENTS_STREAM: &str = "events";
const DEPARTED: &str = "Выехал";
const ARRIVED: &str = "Приехал";
const HELPED_TO_LEAVE: &str = "помог с выездом";
const DAYS: &str = "дн.";

impl Game {
    /// Ехать: маршрут из текущего города, город открыт, проезд по карману, а «по блату» — нашёлся
    /// знакомый, который поможет за услугу. Проезд платится сразу, дни в пути идут дальше.
    pub(super) fn travel(&mut self, destination: &str) -> Result<(), CommandError> {
        let content = Arc::clone(&self.content);
        let to = CityId::new(destination);
        let city = content
            .cities
            .get(&to)
            .ok_or_else(|| CommandError::UnknownCity {
                city: destination.to_owned(),
            })?;
        let route = content
            .routes
            .iter()
            .find(|route| route.from == self.city && route.to == to)
            .ok_or_else(|| CommandError::NoRoute {
                from: self.city.to_string(),
                to: destination.to_owned(),
            })?;
        let today = self.today();
        let access = Access::of(route, &content, today);
        if access == Access::Closed {
            return Err(CommandError::CityClosed {
                city: destination.to_owned(),
                name: city.name.clone(),
            });
        }
        let fare = content.money.from_real(route.fare, Currency::Ruble, today);
        let cash = self.player.money().cash();
        if !cash.covers(fare) {
            return Err(CommandError::NotEnoughCash {
                needed: content.money.format_money(fare, today),
                cash: content.money.format_money(cash, today),
            });
        }
        if access == Access::ByConnections {
            let favour = format!("{HELPED_TO_LEAVE}: {}", city.name);
            let helped = self
                .contacts
                .values_mut()
                .any(|contact| contact.ask(content.reputation, today, &favour).is_ok());
            if !helped {
                return Err(CommandError::NeedsConnections {
                    city: destination.to_owned(),
                    name: city.name.clone(),
                });
            }
        }
        self.player.pay_cash(fare);
        self.journey = Some(Journey {
            from: self.city.clone(),
            to,
            departed: today,
            arrives: today.after_days(route.days),
        });
        let from = &content.cities[&self.city].name;
        let fare = content.money.format_money(fare, today);
        self.log(format!(
            "{DEPARTED}: {from} — {}, {} {DAYS} за {fare}",
            city.name, route.days
        ));
        Ok(())
    }

    /// Маршрут поездки — он есть в контенте, раз по нему уехали.
    fn route_of(&self, journey: &Journey) -> &Route {
        self.content
            .routes
            .iter()
            .find(|route| route.from == journey.from && route.to == journey.to)
            .expect("поездка идёт по маршруту контента")
    }

    /// Дорожное событие дня пути; `true`, если оно случилось.
    pub(super) fn start_road_event(&mut self, date: GameDate, journey: &Journey) -> bool {
        let road_chance = self.content.event_rules.road_daily_chance;
        if !EventEngine::roll(self.rng.stream(EVENTS_STREAM), road_chance) {
            return false;
        }
        let ready = self
            .events
            .ready_road_events(&self.situation(date), &self.route_of(journey).road_events);
        match EventEngine::pick(self.rng.stream(EVENTS_STREAM), &ready) {
            Some(event) => {
                self.events.start(event, date);
                true
            }
            None => false,
        }
    }

    /// Если пришёл последний день пути, игрок в городе назначения; `true` — приехал.
    pub(super) fn arrive_if_due(&mut self, today: GameDate) -> bool {
        let Some(journey) = self.journey.take_if(|journey| today >= journey.arrives) else {
            return false;
        };
        self.city = journey.to;
        let name = &self.content.cities[&self.city].name;
        let message = format!("{ARRIVED}: {name}");
        self.log(message);
        true
    }
}
