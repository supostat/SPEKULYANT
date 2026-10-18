//! Фасад ядра: интерфейс и симулятор видят мир только через `Game`.

mod snapshot;

use std::collections::BTreeMap;
use std::sync::Arc;

use crate::command::{Command, LifeSettings};
use crate::command_error::CommandError;
use crate::content::{BiographyId, CityId, Content, GoodId, MoneyPlaceId, NpcId};
use crate::economy::{Currency, Deal, Markets, Money, Trader};
use crate::reputation::{Contact, FactionReputation};
use crate::rng::WorldRng;
use crate::time::{Clock, GameDate, MAX_SPEED};
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
    messages: Vec<String>,
    screen: Screen,
}

impl Game {
    /// Открывает жизнь в первый день 1984 года: город, наличные и знакомые — из биографии, случай
    /// мира — из сида, репутация у всех фракций нулевая.
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
        Ok(Self {
            clock: Clock::new(content.time.clock.clone(), today),
            rng: WorldRng::new(settings.seed),
            city: biography.start_city.clone(),
            markets: Markets::new(&content),
            player: Trader::new(cash, &content),
            reputation: FactionReputation::new(&content.factions),
            contacts: biography
                .contacts
                .iter()
                .map(|npc| (npc.clone(), Contact::new(npc.clone())))
                .collect(),
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

    /// Двигает мир на `milliseconds` реального времени; `true`, если наступил хотя бы один день.
    /// Каждый день рынки приходят в себя и колеблются по сиду, вклады получают проценты, а
    /// репутация дрейфует к нулю в темпе эпохи.
    pub fn advance(&mut self, milliseconds: u32) -> bool {
        let days = self.clock.advance(milliseconds);
        for day in &days {
            self.markets
                .next_day(&self.content, &mut self.rng, day.date);
            self.player.next_day(&self.content, day.date);
            let drift_days = self
                .content
                .epoch_profile_on(day.date)
                .reputation_drift_days;
            self.reputation.next_day(drift_days);
        }
        !days.is_empty()
    }

    pub fn apply(&mut self, command: Command) -> Result<(), CommandError> {
        let today = self.clock.today();
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
            other => {
                return Err(CommandError::NotSimulatedYet {
                    command: other.name().to_owned(),
                });
            }
        }
        Ok(())
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
