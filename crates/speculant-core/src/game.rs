//! Фасад ядра: интерфейс и симулятор видят мир только через `Game`.

mod business_flow;
mod consequences;
mod event_flow;
mod snapshot;
mod travel_flow;

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::business::{Business, BusinessId};
use crate::command::{Command, LifeSettings};
use crate::command_error::CommandError;
use crate::content::{BiographyId, CityId, Content, GoodId, MoneyPlaceId, NpcId};
use crate::economy::{Currency, Deal, Markets, Money, Trader};
use crate::events::{EventEngine, JournalEntry, Situation};
use crate::npc::Rivals;
use crate::reputation::{Contact, FactionReputation};
use crate::rng::WorldRng;
use crate::time::{Clock, FastForward, GameDate, MAX_SPEED};
use crate::travel::Journey;
use crate::view::{Screen, StateView};

/// Сколько последних сообщений помнит лента.
const MESSAGE_LOG_LENGTH: usize = 20;
const BOUGHT: &str = "Купил";
const SOLD: &str = "Продал";
const DEPOSITED: &str = "положил";
const WITHDREW: &str = "снял";

#[derive(Debug)]
pub struct Game {
    content: Arc<Content>,
    clock: Clock,
    rng: WorldRng,
    city: CityId,
    markets: Markets,
    player: Trader,
    reputation: FactionReputation,
    contacts: BTreeMap<NpcId, Contact>,
    rivals: Rivals,
    businesses: BTreeMap<BusinessId, Business>,
    events: EventEngine,
    journey: Option<Journey>,
    messages: Vec<String>,
    screen: Screen,
}

impl Game {
    /// Открывает жизнь в первый день 1984 года: город, наличные и знакомые — из биографии, случай
    /// мира и состав соперников — из сида, репутация у всех фракций нулевая.
    pub fn new_life(settings: &LifeSettings, content: Arc<Content>) -> Result<Self, CommandError> {
        if settings.biography.trim().is_empty() {
            return Err(CommandError::EmptyBiography);
        }
        let biography = content
            .biographies
            .get(&BiographyId::new(settings.biography.as_str()))
            .ok_or_else(|| CommandError::UnknownBiography {
                biography: settings.biography.clone(),
            })?;
        let today = GameDate::START;
        let cash = content
            .money
            .from_real(biography.start_cash, Currency::Ruble, today);
        let mut rng = WorldRng::new(settings.seed);
        let rivals = Rivals::new(&content, &mut rng, today);
        Ok(Self {
            clock: Clock::new(content.time.clock.clone(), today),
            rng,
            city: biography.start_city.clone(),
            markets: Markets::new(&content),
            player: Trader::new(cash, &content),
            reputation: FactionReputation::new(&content.factions),
            contacts: biography
                .contacts
                .iter()
                .map(|npc| (npc.clone(), Contact::new(npc.clone())))
                .collect(),
            rivals,
            businesses: BTreeMap::new(),
            events: EventEngine::default(),
            journey: None,
            messages: Vec::new(),
            screen: Screen::Market,
            content,
        })
    }

    /// Снимок мира для интерфейса.
    pub fn view(&self) -> StateView {
        snapshot::snapshot(self)
    }

    pub fn today(&self) -> GameDate {
        self.clock.today()
    }

    pub fn content(&self) -> &Content {
        &self.content
    }

    pub fn city(&self) -> &CityId {
        &self.city
    }

    pub fn markets(&self) -> &Markets {
        &self.markets
    }

    pub fn player(&self) -> &Trader {
        &self.player
    }

    pub fn reputation(&self) -> &FactionReputation {
        &self.reputation
    }

    pub fn contacts(&self) -> &BTreeMap<NpcId, Contact> {
        &self.contacts
    }

    /// Соперники мира.
    pub fn rivals(&self) -> &Rivals {
        &self.rivals
    }

    /// Дела игрока.
    pub fn businesses(&self) -> &BTreeMap<BusinessId, Business> {
        &self.businesses
    }

    /// Флаги прошлых решений, ожидающее событие, отложенные продолжения.
    pub fn events(&self) -> &EventEngine {
        &self.events
    }

    /// Журнал событий жизни.
    pub fn journal(&self) -> &[JournalEntry] {
        self.events.journal()
    }

    /// Поездка в пути, если игрок едет.
    pub fn journey(&self) -> Option<&Journey> {
        self.journey.as_ref()
    }

    /// Перемотка, если она идёт.
    pub fn fast_forward(&self) -> Option<FastForward> {
        self.clock.fast_forward()
    }

    /// Двигает мир на `milliseconds` реального времени; `true`, если наступил хотя бы один день.
    /// Каждый день рынки приходят в себя и колеблются по сиду, вклады получают проценты,
    /// репутация дрейфует к нулю в темпе эпохи, соперники и дела живут свой день, а затем может
    /// случиться событие — дома или в дороге. Событие и прибытие останавливают перемотку, и остаток реального времени сгорает;
    /// событие держит время, пока игрок не прочтёт исход.
    pub fn advance(&mut self, milliseconds: u32) -> bool {
        if self.events.pending().is_some() {
            return false;
        }
        self.clock.add_time(milliseconds);
        let mut day_passed = false;
        while let Some(day) = self.clock.next_day() {
            day_passed = true;
            if self.live_day(day.date) {
                self.clock.drop_carried_time();
                self.clock.stop_fast_forward();
                break;
            }
        }
        day_passed
    }

    pub fn apply(&mut self, command: Command) -> Result<(), CommandError> {
        let today = self.clock.today();
        if self.events.pending().is_some() && waits_for_event(&command) {
            return Err(CommandError::EventPending);
        }
        if self.journey.is_some() && needs_a_city(&command) {
            return Err(CommandError::Traveling);
        }
        match command {
            Command::NewLife(settings) => {
                *self = Self::new_life(&settings, Arc::clone(&self.content))?;
            }
            Command::Advance { milliseconds } => {
                self.advance(milliseconds);
            }
            Command::SetSpeed { speed } => {
                self.clock
                    .set_speed(speed)
                    .map_err(|invalid| CommandError::InvalidSpeed {
                        speed: invalid.speed,
                        max: MAX_SPEED,
                    })?;
            }
            Command::Pause { paused } => self.clock.set_paused(paused),
            Command::FastForward { until } => {
                if until == FastForward::UntilArrival && self.journey.is_none() {
                    return Err(CommandError::NotTraveling);
                }
                self.clock.start_fast_forward(until);
            }
            Command::Travel { destination } => self.travel(&destination)?,
            Command::OpenScreen { screen } => self.screen = screen,
            Command::Buy { good, quantity } => {
                let good = GoodId::new(good);
                let deal = self.player.buy(
                    &mut self.markets,
                    &self.content,
                    &self.city,
                    &good,
                    quantity,
                    today,
                )?;
                self.log_deal(BOUGHT, &good, deal);
            }
            Command::Sell { good, quantity } => {
                let good = GoodId::new(good);
                let deal = self.player.sell(
                    &mut self.markets,
                    &self.content,
                    &self.city,
                    &good,
                    quantity,
                    today,
                )?;
                self.log_deal(SOLD, &good, deal);
            }
            Command::Deposit { place, amount } => {
                let place = MoneyPlaceId::new(place);
                let deposited = self.player.deposit(&self.content, &place, amount, today)?;
                self.log_money_move(&place, DEPOSITED, deposited);
            }
            Command::Withdraw { place, amount } => {
                let place = MoneyPlaceId::new(place);
                let withdrawn = self.player.withdraw(&self.content, &place, amount, today)?;
                self.log_money_move(&place, WITHDREW, withdrawn);
            }
            Command::Choose { choice } => self.choose(choice)?,
            Command::Continue => self.close_event()?,
            Command::OpenBusiness { kind } => self.open_business(&kind)?,
            Command::SetPolicy {
                business,
                policy,
                option,
            } => self.set_policy(&business, &policy, option)?,
            Command::Hire {
                business,
                candidate,
            } => self.hire(&business, &candidate)?,
            Command::Fire { business, employee } => self.fire(&business, &employee)?,
            other => {
                return Err(CommandError::NotSimulatedYet {
                    command: other.name().to_owned(),
                });
            }
        }
        Ok(())
    }

    /// День мира; `true`, если он кончился событием или прибытием. Соперники и дела живут свой
    /// день до событий; происшествие с делом, которое вышло за политики, занимает место события
    /// дня. В пути вместо событий дня разыгрываются дорожные события маршрута, а в последний день
    /// пути игрок приезжает — если в этот день случилось событие, то после того, как прочтёт его
    /// исход.
    fn live_day(&mut self, date: GameDate) -> bool {
        self.markets.next_day(&self.content, &mut self.rng, date);
        self.player.next_day(&self.content, date);
        let drift_days = self.content.epoch_profile_on(date).reputation_drift_days;
        self.reputation.next_day(drift_days);
        self.rivals
            .next_day(&self.content, &mut self.markets, &mut self.rng, date);
        if self.live_businesses(date) {
            return true;
        }
        let Some(journey) = self.journey.clone() else {
            return self.start_day_event(date);
        };
        if self.start_road_event(date, &journey) {
            return true;
        }
        self.arrive_if_due(date)
    }

    /// Положение дел для условий событий.
    fn situation(&self, today: GameDate) -> Situation<'_> {
        Situation {
            content: &self.content,
            today,
            city: &self.city,
            on_road: self.journey.is_some(),
            player: &self.player,
            reputation: &self.reputation,
            contacts: &self.contacts,
            rivals: &self.rivals,
            flags: self.events.flags(),
        }
    }

    /// «Купил: Джинсы (фирма), 3 шт за 495 р.»
    fn log_deal(&mut self, verb: &str, good: &GoodId, deal: Deal) {
        let item = &self.content.goods[good];
        let total = self.content.money.format_money(deal.total, self.today());
        let message = format!(
            "{verb}: {}, {} {} за {total}",
            item.name, deal.quantity, item.unit
        );
        self.log(message);
    }

    /// «Сберкасса: положил 500 р.»
    fn log_money_move(&mut self, place: &MoneyPlaceId, verb: &str, amount: Money) {
        let title = &self.content.money_places[place].title;
        let amount = self.content.money.format_money(amount, self.today());
        let message = format!("{title}: {verb} {amount}");
        self.log(message);
    }

    fn log(&mut self, message: String) {
        self.messages.push(message);
        if self.messages.len() > MESSAGE_LOG_LENGTH {
            self.messages.remove(0);
        }
    }
}

/// Пока событие ждёт ответа, мир стоит: торговать, класть деньги и ехать нельзя.
fn waits_for_event(command: &Command) -> bool {
    matches!(
        command,
        Command::Buy { .. }
            | Command::Sell { .. }
            | Command::Deposit { .. }
            | Command::Withdraw { .. }
            | Command::Travel { .. }
            | Command::FastForward { .. }
            | Command::OpenBusiness { .. }
            | Command::SetPolicy { .. }
            | Command::Hire { .. }
            | Command::Fire { .. }
            | Command::Call { .. }
    )
}

/// В дороге нет ни толкучки, ни сберкассы, точку не открыть, и второй поездки не начать.
/// Управлять делами можно и с дороги — по пейджеру.
fn needs_a_city(command: &Command) -> bool {
    matches!(
        command,
        Command::Buy { .. }
            | Command::Sell { .. }
            | Command::Deposit { .. }
            | Command::Withdraw { .. }
            | Command::Travel { .. }
            | Command::OpenBusiness { .. }
    )
}
