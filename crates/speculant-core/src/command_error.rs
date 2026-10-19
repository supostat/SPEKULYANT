//! Отказ ядра выполнить команду — с причиной, которую интерфейс может показать игроку.

use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "error", rename_all = "snake_case")]
pub enum CommandError {
    EmptyBiography,
    UnknownBiography {
        biography: String,
    },
    BiographyLocked {
        biography: String,
        title: String,
        hint: String,
    },
    LifeOver,
    UnknownHousing {
        housing: String,
    },
    AlreadyLivingThere {
        title: String,
    },
    HousingNotOnSale {
        housing: String,
        title: String,
    },
    UnknownEnding {
        ending: String,
    },
    EndingUnavailable {
        ending: String,
        title: String,
    },
    InvalidSpeed {
        speed: u8,
        max: u8,
    },
    UnknownGood {
        good: String,
    },
    NotOnSale {
        good: String,
        name: String,
    },
    ZeroQuantity,
    OutOfStock {
        good: String,
        name: String,
        available: u32,
    },
    NotEnoughCash {
        needed: String,
        cash: String,
    },
    NotInPocket {
        good: String,
        name: String,
        held: u32,
    },
    UnknownPlace {
        place: String,
    },
    ZeroAmount,
    PlaceClosed {
        place: String,
        title: String,
    },
    NotEnoughInPlace {
        place: String,
        title: String,
        balance: String,
    },
    UnknownEvent {
        event: String,
    },
    EventNotReady {
        event: String,
    },
    EventPending,
    NoPendingEvent,
    NoSuchChoice {
        choice: usize,
        count: usize,
    },
    ChoiceUnavailable {
        choice: usize,
    },
    ChoiceAlreadyMade,
    ChoiceRequired,
    UnknownCity {
        city: String,
    },
    NoRoute {
        from: String,
        to: String,
    },
    CityClosed {
        city: String,
        name: String,
    },
    NeedsConnections {
        city: String,
        name: String,
    },
    Traveling,
    NotTraveling,
    InZone,
    NotInZone,
    NoStorage,
    InvalidSlot {
        slot: String,
    },
    IronmanSavesItself,
    IronmanNoRollback,
    NoSuchSave {
        slot: String,
    },
    SaveUnreadable {
        slot: String,
        reason: String,
    },
    SaveFailed {
        reason: String,
    },
    MetaUnreadable {
        reason: String,
    },
    UnknownService {
        service: String,
    },
    StashTooSmall {
        title: String,
        needed: String,
        stash: String,
    },
    ParcelTooSoon {
        from: String,
    },
    ParoleTooEarly {
        from: String,
    },
    NoBribe {
        camp: String,
    },
    UnknownBusinessKind {
        kind: String,
    },
    BusinessNotYet {
        kind: String,
        title: String,
    },
    MaxPoints {
        kind: String,
        title: String,
        max: u32,
    },
    UnknownBusiness {
        business: String,
    },
    UnknownCandidate {
        candidate: String,
    },
    ManagerInPlace {
        business: String,
        name: String,
    },
    CandidateBusy {
        candidate: String,
        name: String,
        business: String,
    },
    CandidateRefuses {
        candidate: String,
        name: String,
        reason: String,
    },
    NotManager {
        business: String,
        employee: String,
    },
    UnknownPolicy {
        business: String,
        policy: String,
    },
    NoSuchPolicyOption {
        policy: String,
        option: usize,
        count: usize,
    },
    NotAContact {
        contact: String,
    },
    ContactLost {
        name: String,
    },
    ContactAway {
        name: String,
        until: String,
    },
}

impl fmt::Display for CommandError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBiography => write!(formatter, "жизнь не начинается без биографии"),
            Self::UnknownBiography { biography } => {
                write!(formatter, "биографии {biography} нет")
            }
            Self::BiographyLocked { title, hint, .. } => {
                write!(formatter, "{title}: пока закрыта — {hint}")
            }
            Self::LifeOver => write!(formatter, "эта жизнь кончилась: начни новую"),
            Self::UnknownHousing { housing } => write!(formatter, "жилья {housing} нет"),
            Self::AlreadyLivingThere { title } => {
                write!(formatter, "{title}: ты и так здесь живёшь")
            }
            Self::HousingNotOnSale { title, .. } => {
                write!(formatter, "{title}: такое жильё сейчас не купить")
            }
            Self::UnknownEnding { ending } => write!(formatter, "финала {ending} нет"),
            Self::EndingUnavailable { title, .. } => {
                write!(formatter, "{title}: так уйти сейчас нельзя")
            }
            Self::InvalidSpeed { speed, max } => {
                write!(formatter, "скорости {speed} нет, есть 1–{max}")
            }
            Self::UnknownGood { good } => write!(formatter, "товара {good} нет"),
            Self::NotOnSale { name, .. } => write!(formatter, "{name}: такого ещё не продают"),
            Self::ZeroQuantity => write!(formatter, "количество должно быть больше нуля"),
            Self::OutOfStock {
                name, available, ..
            } => write!(formatter, "{name}: на рынке только {available}"),
            Self::NotEnoughCash { needed, cash } => {
                write!(
                    formatter,
                    "не хватает денег: нужно {needed}, при себе {cash}"
                )
            }
            Self::NotInPocket { name, held, .. } => {
                write!(formatter, "{name}: в кармане только {held}")
            }
            Self::UnknownPlace { place } => write!(formatter, "места хранения {place} нет"),
            Self::ZeroAmount => write!(formatter, "сумма должна быть больше нуля"),
            Self::PlaceClosed { title, .. } => {
                write!(formatter, "{title}: сейчас туда не положить")
            }
            Self::NotEnoughInPlace { title, balance, .. } => {
                write!(formatter, "{title}: там только {balance}")
            }
            Self::UnknownEvent { event } => write!(formatter, "события {event} нет"),
            Self::EventNotReady { event } => {
                write!(formatter, "событие {event} сейчас случиться не может")
            }
            Self::EventPending => write!(formatter, "время стоит: сначала ответь на событие"),
            Self::NoPendingEvent => write!(formatter, "сейчас ничего не происходит"),
            Self::NoSuchChoice { choice, count } => {
                write!(formatter, "выбора {choice} нет, есть 1–{count}")
            }
            Self::ChoiceUnavailable { choice } => {
                write!(formatter, "выбор {choice} сейчас недоступен")
            }
            Self::ChoiceAlreadyMade => write!(formatter, "выбор уже сделан"),
            Self::ChoiceRequired => write!(formatter, "сначала выбери, что делать"),
            Self::UnknownCity { city } => write!(formatter, "города {city} нет"),
            Self::NoRoute { from, to } => write!(formatter, "из {from} в {to} не ездят"),
            Self::CityClosed { name, .. } => write!(formatter, "{name}: туда пока не пускают"),
            Self::NeedsConnections { name, .. } => {
                write!(formatter, "{name}: только по блату, а помочь некому")
            }
            Self::Traveling => write!(formatter, "ты в дороге"),
            Self::NotTraveling => write!(formatter, "ты никуда не едешь"),
            Self::InZone => write!(formatter, "ты в зоне"),
            Self::NotInZone => write!(formatter, "ты на воле"),
            Self::NoStorage => write!(formatter, "сохранять некуда: у игры нет каталога игрока"),
            Self::InvalidSlot { slot } => write!(
                formatter,
                "«{slot}» — не имя слота: латиница в нижнем регистре, цифры и _, до 32 знаков, и не ironman"
            ),
            Self::IronmanSavesItself => {
                write!(formatter, "в ironman жизнь сохраняется сама")
            }
            Self::IronmanNoRollback => write!(formatter, "в ironman отката нет"),
            Self::NoSuchSave { slot } => write!(formatter, "в слоте {slot} ничего нет"),
            Self::SaveUnreadable { slot, reason } => {
                write!(formatter, "слот {slot} не прочесть: {reason}")
            }
            Self::SaveFailed { reason } => write!(formatter, "сохранить не вышло: {reason}"),
            Self::MetaUnreadable { reason } => {
                write!(formatter, "знания прошлых жизней не прочесть: {reason}")
            }
            Self::UnknownService { service } => write!(formatter, "услуги {service} в зоне нет"),
            Self::StashTooSmall {
                title,
                needed,
                stash,
            } => write!(formatter, "{title}: нужно {needed}, а у тебя {stash}"),
            Self::ParcelTooSoon { from } => write!(formatter, "следующая передача — с {from}"),
            Self::ParoleTooEarly { from } => write!(formatter, "просить об УДО можно с {from}"),
            Self::NoBribe { camp } => write!(formatter, "{camp}: здесь на волю не продают"),
            Self::UnknownBusinessKind { kind } => write!(formatter, "дела {kind} не бывает"),
            Self::BusinessNotYet { title, .. } => {
                write!(formatter, "{title}: сейчас такое дело не открыть")
            }
            Self::MaxPoints { title, max, .. } => {
                write!(formatter, "{title}: больше {max} точек в городе не держат")
            }
            Self::UnknownBusiness { business } => write!(formatter, "дела {business} у тебя нет"),
            Self::UnknownCandidate { candidate } => {
                write!(formatter, "{candidate} в управляющие не нанимается")
            }
            Self::ManagerInPlace { business, name } => {
                write!(formatter, "{business}: управляющий уже есть — {name}")
            }
            Self::CandidateBusy { name, business, .. } => {
                write!(formatter, "{name} уже управляет делом «{business}»")
            }
            Self::CandidateRefuses { name, reason, .. } => write!(formatter, "{name}: {reason}"),
            Self::NotManager { business, employee } => {
                write!(formatter, "{employee} не управляет делом {business}")
            }
            Self::UnknownPolicy { business, policy } => {
                write!(formatter, "у дела {business} нет политики {policy}")
            }
            Self::NoSuchPolicyOption {
                policy,
                option,
                count,
            } => write!(
                formatter,
                "у политики {policy} нет варианта {option}: вариантов {count}, с нуля"
            ),
            Self::NotAContact { contact } => write!(formatter, "{contact} нет среди связей"),
            Self::ContactLost { name } => write!(formatter, "{name}: связь потеряна"),
            Self::ContactAway { name, until } => {
                write!(formatter, "{name} не на связи до {until}")
            }
        }
    }
}

impl std::error::Error for CommandError {}
