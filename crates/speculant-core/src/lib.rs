//! Ядро симуляции «Спекулянта»: время, экономика, репутация, события, поездки, соперники, дела.
//! Крейт не зависит от Godot: его гоняют симулятор, валидатор и мост gdext.
//! Наружу ядро отдаёт `StateView` — снимок мира, готовый к показу, — и принимает `Command`.

pub mod command;
pub mod command_error;
pub mod content;
pub mod economy;
pub mod events;
pub mod game;
pub mod reputation;
pub mod rng;
pub mod time;
pub mod travel;
pub mod view;

pub use command::{Command, LifeSettings};
pub use command_error::CommandError;
pub use content::Content;
pub use game::Game;
pub use time::TimeRules;
pub use view::StateView;

/// Версия ядра из манифеста рабочей области.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::VERSION;

    #[test]
    fn version_is_a_semver_triplet() {
        let parts: Vec<u32> = VERSION
            .split('.')
            .map(|part| part.parse().expect("числовая часть версии"))
            .collect();
        assert_eq!(parts.len(), 3);
    }
}
