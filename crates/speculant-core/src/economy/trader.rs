use serde::{Deserialize, Serialize};

use super::{Currency, Markets, Money, MoneyStore, Pocket};
use crate::command_error::CommandError;
use crate::content::{CityId, Content, Good, GoodId, MoneyPlace, MoneyPlaceId};
use crate::time::GameDate;

/// Сделка на толкучке: сколько штук и за какую сумму всего.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Deal {
    pub quantity: u32,
    pub total: Money,
}

/// Торговец — игрок, а позже и соперники (§8): деньги, карман и сделки по общим правилам рынка.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Trader {
    money: MoneyStore,
    pocket: Pocket,
}

impl Trader {
    /// Торговец с наличными рублями и пустыми вкладами во всех местах хранения контента.
    pub fn new(cash: Money, content: &Content) -> Self {
        Self {
            money: MoneyStore::new(cash, &content.money_places),
            pocket: Pocket::default(),
        }
    }

    pub fn money(&self) -> &MoneyStore {
        &self.money
    }

    pub fn pocket(&self) -> &Pocket {
        &self.pocket
    }

    /// Деньги от события: выручка, находка, подарок.
    pub fn receive_cash(&mut self, amount: Money) {
        self.money.receive(amount);
    }

    /// Деньги на событие; хватает ли их, проверяет доступность выбора.
    pub fn pay_cash(&mut self, amount: Money) {
        self.money.pay(amount);
    }

    /// Деньги пропали из места хранения — кража, пожар, лопнувший банк; больше, чем лежит, не
    /// пропадает — это проверяет вызывающий.
    pub fn lose_from(&mut self, place: &MoneyPlaceId, amount: Money) {
        self.money.take(place, amount);
    }

    /// Товар от события.
    pub fn add_goods(&mut self, good: &GoodId, quantity: u32) {
        self.pocket.add(good, quantity);
    }

    /// Товар на событие; есть ли он в кармане, проверяет доступность выбора.
    pub fn remove_goods(&mut self, good: &GoodId, quantity: u32) {
        self.pocket.remove(good, quantity);
    }

    /// Покупка на толкучке города: каждая следующая штука дороже, запас тает, платят наличными.
    pub fn buy(
        &mut self,
        markets: &mut Markets,
        content: &Content,
        city: &CityId,
        good: &GoodId,
        quantity: u32,
        date: GameDate,
    ) -> Result<Deal, CommandError> {
        let item = known_good(content, good)?;
        if quantity == 0 {
            return Err(CommandError::ZeroQuantity);
        }
        if !Markets::is_on_sale(content, good, date) {
            return Err(not_on_sale(good, item));
        }
        let available = markets.city(city).stock(good);
        if quantity > available {
            return Err(CommandError::OutOfStock {
                good: good.to_string(),
                name: item.name.clone(),
                available,
            });
        }
        let tags = markets
            .buy_tags(content, city, good, quantity, date)
            .expect("товар в продаже — проверено выше");
        let total = sum(&tags);
        self.ensure_cash(content, total, date)?;
        markets.take(city, good, quantity);
        self.money.pay(total);
        self.pocket.add(good, quantity);
        Ok(Deal { quantity, total })
    }

    /// Продажа на толкучке города: рынок заваливается, каждая следующая штука дешевле.
    pub fn sell(
        &mut self,
        markets: &mut Markets,
        content: &Content,
        city: &CityId,
        good: &GoodId,
        quantity: u32,
        date: GameDate,
    ) -> Result<Deal, CommandError> {
        let item = known_good(content, good)?;
        if quantity == 0 {
            return Err(CommandError::ZeroQuantity);
        }
        let held = self.pocket.quantity(good);
        if quantity > held {
            return Err(CommandError::NotInPocket {
                good: good.to_string(),
                name: item.name.clone(),
                held,
            });
        }
        let tags = markets
            .sell_tags(content, city, good, quantity, date)
            .expect("товар в кармане куплен, значит уже в продаже");
        let total = sum(&tags);
        markets.put(city, good, quantity);
        self.money.receive(total);
        self.pocket.remove(good, quantity);
        Ok(Deal { quantity, total })
    }

    /// Кладёт `units` целых единиц валюты места: рубли — из наличных, чеки и доллары — купленные
    /// на наличные по курсу дня. Возвращает положенную сумму в валюте места.
    pub fn deposit(
        &mut self,
        content: &Content,
        place: &MoneyPlaceId,
        units: u64,
        date: GameDate,
    ) -> Result<Money, CommandError> {
        let spot = known_place(content, place)?;
        if units == 0 {
            return Err(CommandError::ZeroAmount);
        }
        if !spot.accepts_deposits_on(date) {
            return Err(CommandError::PlaceClosed {
                place: place.to_string(),
                title: spot.title.clone(),
            });
        }
        let amount = content.money.whole_units(spot.currency, units, date);
        let cost = content.money.exchange(amount, Currency::Ruble, date);
        self.ensure_cash(content, cost, date)?;
        self.money.pay(cost);
        self.money.put(place, amount);
        Ok(amount)
    }

    /// Забирает `units` целых единиц валюты места в наличные рубли по курсу дня. Возвращает
    /// забранную сумму в валюте места.
    pub fn withdraw(
        &mut self,
        content: &Content,
        place: &MoneyPlaceId,
        units: u64,
        date: GameDate,
    ) -> Result<Money, CommandError> {
        let spot = known_place(content, place)?;
        if units == 0 {
            return Err(CommandError::ZeroAmount);
        }
        let amount = content.money.whole_units(spot.currency, units, date);
        let balance = self.money.balance(place);
        if !balance.covers(amount) {
            return Err(CommandError::NotEnoughInPlace {
                place: place.to_string(),
                title: spot.title.clone(),
                balance: content.money.format_money(balance, date),
            });
        }
        self.money.take(place, amount);
        self.money
            .receive(content.money.exchange(amount, Currency::Ruble, date));
        Ok(amount)
    }

    /// Новый день: первого числа места хранения начисляют проценты за месяц.
    pub fn next_day(&mut self, content: &Content, date: GameDate) {
        if date.day_of_month() == 1 {
            self.money
                .accrue_monthly_interest(&content.money_places, date);
        }
    }

    fn ensure_cash(
        &self,
        content: &Content,
        needed: Money,
        date: GameDate,
    ) -> Result<(), CommandError> {
        let cash = self.money.cash();
        if cash.covers(needed) {
            Ok(())
        } else {
            Err(CommandError::NotEnoughCash {
                needed: content.money.format_money(needed, date),
                cash: content.money.format_money(cash, date),
            })
        }
    }
}

fn known_good<'a>(content: &'a Content, good: &GoodId) -> Result<&'a Good, CommandError> {
    content
        .goods
        .get(good)
        .ok_or_else(|| CommandError::UnknownGood {
            good: good.to_string(),
        })
}

fn known_place<'a>(
    content: &'a Content,
    place: &MoneyPlaceId,
) -> Result<&'a MoneyPlace, CommandError> {
    content
        .money_places
        .get(place)
        .ok_or_else(|| CommandError::UnknownPlace {
            place: place.to_string(),
        })
}

fn not_on_sale(good: &GoodId, item: &Good) -> CommandError {
    CommandError::NotOnSale {
        good: good.to_string(),
        name: item.name.clone(),
    }
}

fn sum(tags: &[Money]) -> Money {
    tags.iter()
        .fold(Money::zero(Currency::Ruble), |total, tag| total.plus(*tag))
}
