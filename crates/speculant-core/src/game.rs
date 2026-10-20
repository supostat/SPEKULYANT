//! Фасад ядра: интерфейс и симулятор видят мир только через `Game`.

mod business_flow;
mod call_flow;
mod consequences;
mod event_flow;
mod history_flow;
mod life_flow;
mod lobby;
mod meta_flow;
mod russian_count;
mod save_flow;
mod snapshot;
mod travel_flow;
mod world;
mod zone_flow;

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::business::{Business, BusinessId};
use crate::command::{Command, LifeSettings};
use crate::command_error::CommandError;
use crate::content::{BiographyId, CityId, Content, GoodId, MoneyPlaceId, NpcId};
use crate::economy::{Currency, Deal, Markets, Money, PriceHistory, Real, Trader};
use crate::events::{EventEngine, JournalEntry, Situation};
use crate::history::Timeline;
use crate::life::{Character, Chronicle, LifeSummary, Pager, PagerDevice, PagerMessage};
use crate::meta::Meta;
use crate::npc::Rivals;
use crate::reputation::{Contact, FactionReputation};
use crate::rng::WorldRng;
use crate::save::Storage;
use crate::time::{Clock, FastForward, GameDate, MAX_SPEED};
use crate::travel::Journey;
use crate::view::{Screen, StateView};
use crate::zone::Zone;
use world::World;

pub use lobby::Lobby;

/// Сколько последних сообщений помнит лента.
const MESSAGE_LOG_LENGTH: usize = 20;
const BOUGHT: &str = "Купил";
const SOLD: &str = "Продал";
const DEPOSITED: &str = "положил";
const WITHDREW: &str = "снял";
const LIFE_STREAM: &str = "life";
const HISTORY_STREAM: &str = "history";

/// Игра: данные, метапрогрессия прошлых жизней, каталог игрока, если он есть, и мир этой жизни.
#[derive(Debug)]
pub struct Game {
    content: Arc<Content>,
    meta: Meta,
    /// Без каталога игра живёт в памяти: так её гоняют симулятор и тесты.
    storage: Option<Storage>,
    world: World,
}

impl Game {
    /// Первая жизнь сеанса в памяти, без каталога игрока: доступны только биографии, открытые с
    /// начала.
    pub fn new_life(settings: &LifeSettings, content: Arc<Content>) -> Result<Self, CommandError> {
        Self::start_life(settings, content, Meta::default(), None)
    }

    /// Жизнь по биографии, открытой с начала или прошлыми жизнями: город, наличные, знакомые и
    /// жильё — из биографии, имя, возраст, случай мира, история и состав соперников — из сида,
    /// репутация у всех фракций нулевая. Ironman-жизнь сразу ложится в свой слот.
    fn start_life(
        settings: &LifeSettings,
        content: Arc<Content>,
        meta: Meta,
        storage: Option<Storage>,
    ) -> Result<Self, CommandError> {
        if settings.biography.trim().is_empty() {
            return Err(CommandError::EmptyBiography);
        }
        let id = BiographyId::new(settings.biography.as_str());
        let biography =
            content
                .biographies
                .get(&id)
                .ok_or_else(|| CommandError::UnknownBiography {
                    biography: settings.biography.clone(),
                })?;
        if !biography.starts_unlocked && !meta.unlocked().contains(&id) {
            return Err(CommandError::BiographyLocked {
                biography: settings.biography.clone(),
                title: biography.title.clone(),
                hint: biography.unlock_hint.clone().unwrap_or_default(),
            });
        }
        let today = GameDate::START;
        let cash = content
            .money
            .from_real(biography.start_cash, Currency::Ruble, today);
        let mut rng = WorldRng::new(settings.seed);
        let rivals = Rivals::new(&content, &mut rng, today);
        let timeline = Timeline::generate(&content, rng.stream(HISTORY_STREAM));
        let character =
            life_flow::born_character(&content, &id, biography, rng.stream(LIFE_STREAM));
        let world = World {
            ironman: settings.ironman,
            character,
            chronicle: Chronicle::default(),
            summary: None,
            clock: Clock::new(content.time.clock.clone(), today),
            rng,
            timeline,
            city: biography.start_city.clone(),
            markets: Markets::new(&content),
            price_history: PriceHistory::default(),
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
            zone: None,
            messages: Vec::new(),
            pager: Pager::default(),
            screen: Screen::Market,
        };
        let mut game = Self {
            content,
            meta,
            storage,
            world,
        };
        game.record_prices(today);
        game.autosave();
        Ok(game)
    }

    /// Снимок мира для интерфейса.
    pub fn view(&self) -> StateView {
        snapshot::snapshot(self)
    }

    pub fn today(&self) -> GameDate {
        self.world.clock.today()
    }

    pub fn content(&self) -> &Content {
        &self.content
    }

    pub fn city(&self) -> &CityId {
        &self.world.city
    }

    pub fn markets(&self) -> &Markets {
        &self.world.markets
    }

    pub fn player(&self) -> &Trader {
        &self.world.player
    }

    pub fn reputation(&self) -> &FactionReputation {
        &self.world.reputation
    }

    pub fn contacts(&self) -> &BTreeMap<NpcId, Contact> {
        &self.world.contacts
    }

    /// Соперники мира.
    pub fn rivals(&self) -> &Rivals {
        &self.world.rivals
    }

    /// Персонаж этой жизни.
    pub fn character(&self) -> &Character {
        &self.world.character
    }

    /// Знания и открытия прошлых жизней.
    pub fn meta(&self) -> &Meta {
        &self.meta
    }

    /// Ironman-жизнь сохраняется сама и не откатывается.
    pub fn is_ironman(&self) -> bool {
        self.world.ironman
    }

    /// Капитал сегодня в рублях 1984 года — тот, что итоги запишут, если жизнь кончится сейчас.
    pub fn capital(&self) -> Real {
        self.capital_on(self.today())
    }

    /// Итоги, если жизнь кончилась.
    pub fn summary(&self) -> Option<&LifeSummary> {
        self.world.summary.as_ref()
    }

    /// Дела игрока.
    pub fn businesses(&self) -> &BTreeMap<BusinessId, Business> {
        &self.world.businesses
    }

    /// Флаги прошлых решений, ожидающее событие, отложенные продолжения.
    pub fn events(&self) -> &EventEngine {
        &self.world.events
    }

    /// Журнал событий жизни.
    pub fn journal(&self) -> &[JournalEntry] {
        self.world.events.journal()
    }

    /// Поездка в пути, если игрок едет.
    pub fn journey(&self) -> Option<&Journey> {
        self.world.journey.as_ref()
    }

    /// История мира по сиду.
    pub fn timeline(&self) -> &Timeline {
        &self.world.timeline
    }

    /// Отсидка, если игрок в зоне.
    pub fn zone(&self) -> Option<&Zone> {
        self.world.zone.as_ref()
    }

    /// Перемотка, если она идёт.
    pub fn fast_forward(&self) -> Option<FastForward> {
        self.world.clock.fast_forward()
    }

    /// Двигает мир на `milliseconds` реального времени; `true`, если наступил хотя бы один день.
    /// Каждый день рынки приходят в себя и колеблются по сиду, вклады получают проценты,
    /// репутация дрейфует к нулю в темпе эпохи, соперники живут свой день, случаются узлы истории
    /// этого дня, персонаж и дела живут свой день, а затем может
    /// случиться событие — дома, в дороге или в зоне, где неделя идёт за время одного дня.
    /// Событие, прибытие и выход на волю останавливают перемотку, и остаток реального времени
    /// сгорает; событие держит время, пока игрок не прочтёт исход.
    pub fn advance(&mut self, milliseconds: u32) -> bool {
        if self.world.events.pending().is_some() || self.world.summary.is_some() {
            return false;
        }
        self.world.clock.add_time(milliseconds);
        let mut day_passed = false;
        while let Some(day) = self.world.clock.next_day() {
            day_passed = true;
            if self.live_day(day.date) {
                self.world.clock.drop_carried_time();
                self.world.clock.stop_fast_forward();
                break;
            }
        }
        if day_passed {
            self.autosave();
        }
        day_passed
    }

    pub fn apply(&mut self, command: Command) -> Result<(), CommandError> {
        let today = self.world.clock.today();
        if self.world.events.pending().is_some() && changes_the_world(&command) {
            return Err(CommandError::EventPending);
        }
        if self.world.summary.is_some() && changes_the_world(&command) {
            return Err(CommandError::LifeOver);
        }
        if self.world.journey.is_some() && needs_a_city(&command) {
            return Err(CommandError::Traveling);
        }
        if self.world.zone.is_some() && needs_freedom(&command) {
            return Err(CommandError::InZone);
        }
        let deed = is_a_deed(&command);
        match command {
            Command::NewLife(settings) => {
                *self = Self::start_life(
                    &settings,
                    Arc::clone(&self.content),
                    self.meta.clone(),
                    self.storage.clone(),
                )?;
            }
            Command::Advance { milliseconds } => {
                self.advance(milliseconds);
            }
            Command::SetSpeed { speed } => {
                self.world.clock.set_speed(speed).map_err(|invalid| {
                    CommandError::InvalidSpeed {
                        speed: invalid.speed,
                        max: MAX_SPEED,
                    }
                })?;
            }
            Command::Pause { paused } => self.world.clock.set_paused(paused),
            Command::FastForward { until } => {
                if until == FastForward::UntilArrival && self.world.journey.is_none() {
                    return Err(CommandError::NotTraveling);
                }
                self.world.clock.start_fast_forward(until);
            }
            Command::Travel { destination } => self.travel(&destination)?,
            Command::OpenScreen { screen } => self.world.screen = screen,
            Command::Buy { good, quantity } => {
                let good = GoodId::new(good);
                let deal = self.world.player.buy(
                    &mut self.world.markets,
                    &self.content,
                    &self.world.city,
                    &good,
                    quantity,
                    today,
                )?;
                let cost = self.content.money.real_value(deal.total, today);
                self.world
                    .chronicle
                    .bought(&good, deal.quantity, cost, today, &self.world.city);
                self.log_deal(BOUGHT, &good, deal);
            }
            Command::Sell { good, quantity } => {
                let good = GoodId::new(good);
                let held = self.world.player.pocket().quantity(&good);
                let deal = self.world.player.sell(
                    &mut self.world.markets,
                    &self.content,
                    &self.world.city,
                    &good,
                    quantity,
                    today,
                )?;
                let proceeds = self.content.money.real_value(deal.total, today);
                self.world.chronicle.sold(
                    &good,
                    deal.quantity,
                    held,
                    proceeds,
                    today,
                    &self.world.city,
                );
                self.log_deal(SOLD, &good, deal);
            }
            Command::Deposit { place, amount } => {
                let place = MoneyPlaceId::new(place);
                let deposited = self
                    .world
                    .player
                    .deposit(&self.content, &place, amount, today)?;
                self.log_money_move(&place, DEPOSITED, deposited);
            }
            Command::Withdraw { place, amount } => {
                let place = MoneyPlaceId::new(place);
                let withdrawn = self
                    .world
                    .player
                    .withdraw(&self.content, &place, amount, today)?;
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
            Command::BuyHousing { housing } => self.buy_housing(&housing)?,
            Command::Retire { ending } => self.retire(&ending)?,
            Command::SetWork { work } => self.set_work(work)?,
            Command::UseService { service } => self.use_service(&service)?,
            Command::OrderParcel => self.order_parcel()?,
            Command::ApplyForParole => self.apply_for_parole()?,
            Command::BribeForRelease => self.bribe_for_release()?,
            Command::Call { contact } => self.call(&contact)?,
            Command::ErasePagerMessage { message } => {
                if !self.world.pager.erase(message) {
                    return Err(CommandError::NoSuchPagerMessage {
                        message,
                        count: self.world.pager.messages().len(),
                    });
                }
            }
            Command::Save { slot } => self.save(&slot)?,
            Command::Load { slot } => self.load(&slot)?,
            Command::Quit => self.autosave(),
        }
        if deed {
            self.autosave();
        }
        Ok(())
    }

    /// День мира; `true`, если он кончился событием, прибытием или выходом на волю. Соперники и
    /// дела живут свой день до событий; происшествие с делом, которое вышло за политики, занимает
    /// место события дня — оно находит хозяина и в зоне. В зоне вместо событий дня живёт зона. В
    /// пути вместо событий дня разыгрываются дорожные события маршрута, а в последний день пути
    /// игрок приезжает — если в этот день случилось событие, то после того, как прочтёт его исход.
    fn live_day(&mut self, date: GameDate) -> bool {
        self.world
            .markets
            .next_day(&self.content, &mut self.world.rng, date);
        if PriceHistory::is_record_day(date) {
            self.record_prices(date);
        }
        self.world.player.next_day(&self.content, date);
        let drift_days = self.content.epoch_profile_on(date).reputation_drift_days;
        self.world.reputation.next_day(drift_days);
        self.world.rivals.next_day(
            &self.content,
            &mut self.world.markets,
            &mut self.world.rng,
            date,
        );
        self.live_history(date);
        if self.live_character(date) || self.live_businesses(date) {
            return true;
        }
        if self.world.zone.is_some() {
            return self.live_zone_day(date);
        }
        let Some(journey) = self.world.journey.clone() else {
            return self.start_day_event(date);
        };
        if self.start_road_event(date, &journey) {
            return true;
        }
        self.arrive_if_due(date)
    }

    /// Неделя истории цен: цена покупки штуки каждого товара в каждом городе в этот день.
    fn record_prices(&mut self, date: GameDate) {
        let content = &self.content;
        let markets = &self.world.markets;
        let prices = content.cities.keys().flat_map(|city| {
            content.goods.keys().map(move |good| {
                let kopecks = markets.buy_real_kopecks(content, city, good, date);
                (city, good, kopecks.map(|kopecks| kopecks.round() as u32))
            })
        });
        self.world.price_history.record(date, prices);
    }

    /// Положение дел для условий событий.
    fn situation(&self, today: GameDate) -> Situation<'_> {
        Situation {
            content: &self.content,
            today,
            city: &self.world.city,
            on_road: self.world.journey.is_some(),
            player: &self.world.player,
            reputation: &self.world.reputation,
            contacts: &self.world.contacts,
            rivals: &self.world.rivals,
            flags: self.world.events.flags(),
            timeline: &self.world.timeline,
            zone: self.world.zone.as_ref(),
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
        self.world.messages.push(message);
        if self.world.messages.len() > MESSAGE_LOG_LENGTH {
            self.world.messages.remove(0);
        }
    }

    /// Сообщение на пейджер — дела, крыша, рынок, потери, — если устройство при игроке; без
    /// устройства оно проходит мимо.
    fn page(&mut self, text: String, urgent: bool) {
        if self.pager_device().is_none() {
            return;
        }
        let message = PagerMessage {
            date: self.today(),
            text,
            urgent,
        };
        self.world
            .pager
            .record(message, self.content.life.pager.messages);
    }

    /// Устройство пейджера — первое по правилам, что лежит в кармане.
    fn pager_device(&self) -> Option<&PagerDevice> {
        self.content
            .life
            .pager
            .devices
            .iter()
            .find(|device| self.world.player.pocket().quantity(&device.good) > 0)
    }
}

/// Команды, которые меняют мир: пока событие ждёт ответа, их не выполнить, а когда жизнь
/// кончилась, остаются только экраны, исход последнего события и новая жизнь.
fn changes_the_world(command: &Command) -> bool {
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
            | Command::BuyHousing { .. }
            | Command::Retire { .. }
            | Command::SetWork { .. }
            | Command::UseService { .. }
            | Command::OrderParcel
            | Command::ApplyForParole
            | Command::BribeForRelease
    )
}

/// В дороге нет ни толкучки, ни сберкассы, точку и жильё не купить, на покой не уйти, и второй
/// поездки не начать. Управлять делами можно и с дороги — по пейджеру.
fn needs_a_city(command: &Command) -> bool {
    matches!(
        command,
        Command::Buy { .. }
            | Command::Sell { .. }
            | Command::Deposit { .. }
            | Command::Withdraw { .. }
            | Command::Travel { .. }
            | Command::OpenBusiness { .. }
            | Command::BuyHousing { .. }
            | Command::Retire { .. }
    )
}

/// Из зоны не поторгуешь, не съездишь, денег не положишь и не снимешь, делами не распорядишься,
/// никому не позвонишь, жилья не купишь и на покой не уйдёшь.
fn needs_freedom(command: &Command) -> bool {
    matches!(
        command,
        Command::Buy { .. }
            | Command::Sell { .. }
            | Command::Deposit { .. }
            | Command::Withdraw { .. }
            | Command::Travel { .. }
            | Command::OpenBusiness { .. }
            | Command::SetPolicy { .. }
            | Command::Hire { .. }
            | Command::Fire { .. }
            | Command::Call { .. }
            | Command::BuyHousing { .. }
            | Command::Retire { .. }
    )
}

/// Поступки, после которых ironman-жизнь сохраняется сама: всё, кроме экранов, скорости, паузы,
/// сохранений и новой жизни, которая сохраняется при рождении, — а время сохраняется по дням.
fn is_a_deed(command: &Command) -> bool {
    !matches!(
        command,
        Command::NewLife(_)
            | Command::Advance { .. }
            | Command::SetSpeed { .. }
            | Command::Pause { .. }
            | Command::OpenScreen { .. }
            | Command::Save { .. }
            | Command::Load { .. }
            | Command::Quit
    )
}
