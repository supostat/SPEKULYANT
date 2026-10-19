//! Свои дела бота-предпринимателя: точка, когда наличных хватает с запасом, и управляющий в
//! каждое дело без него.

use speculant_core::business::BusinessId;
use speculant_core::economy::Currency;
use speculant_core::{Command, Game};

/// Во сколько раз наличные должны перекрывать цену точки: остальное — оборотные на торговлю.
const CASH_OVER_POINT_COST: f64 = 1.5;

pub(crate) fn expand(game: &mut Game) {
    open_point(game);
    hire_managers(game);
}

/// Самая дорогая точка, которую можно открыть в городе сегодня с запасом наличных.
fn open_point(game: &mut Game) {
    let content = game.content();
    let today = game.today();
    let cash = game.player().money().cash().minor_units() as f64;
    let affordable = content
        .business_kinds
        .iter()
        .filter(|(_, kind)| kind.form_on(&content.business_forms, today).is_some())
        .filter(|(id, kind)| {
            let business = BusinessId::new((*id).clone(), game.city().clone());
            game.businesses()
                .get(&business)
                .map_or(0, |business| business.points())
                < kind.max_points
        })
        .filter(|(_, kind)| {
            let cost = content
                .money
                .from_real(kind.point_cost, Currency::Ruble, today);
            cash >= cost.minor_units() as f64 * CASH_OVER_POINT_COST
        })
        .max_by_key(|(_, kind)| kind.point_cost.kopecks())
        .map(|(id, _)| id.to_string());
    if let Some(kind) = affordable {
        game.apply(Command::OpenBusiness { kind })
            .expect("форма открыта, точек меньше предела, наличных хватает");
    }
}

/// В каждое дело без управляющего — первого из кандидатов, если они есть.
fn hire_managers(game: &mut Game) {
    let unmanaged: Vec<BusinessId> = game
        .businesses()
        .iter()
        .filter(|(_, business)| business.manager().is_none())
        .map(|(id, _)| id.clone())
        .collect();
    for business in unmanaged {
        let Some(candidate) = game.view().candidates.into_iter().next() else {
            return;
        };
        game.apply(Command::Hire {
            business: business.to_string(),
            candidate: candidate.candidate,
        })
        .expect("кандидат из списка свободен и не обижен");
    }
}

#[cfg(test)]
mod tests {
    use speculant_core::business::BusinessId;

    use super::expand;
    use crate::test_worlds::{core_fixture, new_life};

    fn business(id: &str) -> BusinessId {
        BusinessId::parse(id).expect("дело вида тип@город")
    }

    #[test]
    fn the_entrepreneur_opens_the_dearest_point_he_can_and_puts_a_manager_in() {
        let mut game = new_life(&core_fixture("business"), "student");
        expand(&mut game);
        let workshop = &game.businesses()[&business("sewing_workshop@moscow")];
        assert_eq!(
            workshop.points(),
            1,
            "в 1984-м открыт только подпольный цех и артель"
        );
        let manager = workshop.manager().expect("управляющий нанят");
        assert_eq!(manager.employee().npc().as_str(), "gena_kabanov");

        expand(&mut game);
        assert_eq!(
            game.businesses().len(),
            2,
            "цех — одна точка, дальше палатка"
        );
        assert!(
            game.businesses()[&business("stall@moscow")]
                .manager()
                .is_some()
        );
    }
}
