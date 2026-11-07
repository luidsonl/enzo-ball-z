//! Mecânicas que **não** são slots: andar/correr, SixSeven, dano de contato e
//! timers genéricos.
//!
//! `Locomotion` reúne o que existe fora do loadout. Pular, dash, voar, soco,
//! Mogging e qualquer especial vivem em [`crate::skills`].

pub mod combat;
pub mod sixseven;
pub mod walk;

pub use combat::{CONTACT_IFRAMES, PLAYER_KNOCKBACK, PLAYER_KNOCKBACK_LIFT};
pub use sixseven::SixSeven;
pub use walk::Walk;

use crate::input::{PlayerCommand, SixSevenKey};

/// Estado contínuo de locomoção de uma entidade.
///
/// `walk_enabled`/`sixseven_enabled` dizem quais mecânicas a entidade possui.
/// Inimigos andam, mas não farmam aura; projéteis e itens não têm locomoção.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Locomotion {
    pub walk: Walk,
    pub sixseven: SixSeven,
    pub walk_enabled: bool,
    pub sixseven_enabled: bool,
}

impl Default for Locomotion {
    fn default() -> Self {
        Self::none()
    }
}

impl Locomotion {
    pub const fn none() -> Self {
        Self {
            walk: Walk::new(),
            sixseven: SixSeven::new(),
            walk_enabled: false,
            sixseven_enabled: false,
        }
    }

    pub const fn player() -> Self {
        Self {
            walk: Walk::new(),
            sixseven: SixSeven::new(),
            walk_enabled: true,
            sixseven_enabled: true,
        }
    }

    /// Inimigo anda, mas não tem SixSeven.
    pub const fn enemy() -> Self {
        Self {
            walk: Walk::new(),
            sixseven: SixSeven::new(),
            walk_enabled: true,
            sixseven_enabled: false,
        }
    }

    pub fn has_walk(self) -> bool {
        self.walk_enabled
    }

    pub fn has_sixseven(self) -> bool {
        self.sixseven_enabled
    }

    /// Registra a intenção contínua e edges das mecânicas fora dos slots.
    pub fn apply_command(&mut self, command: PlayerCommand) {
        if self.walk_enabled {
            self.walk.apply(command.move_x);
        }
        if self.sixseven_enabled {
            if let Some(key) = command.sixseven {
                self.sixseven.request(key);
            }
        }
    }

    /// Avança janelas internas de andar e SixSeven.
    pub fn tick(&mut self, dt: f32) {
        if self.walk_enabled {
            self.walk.tick(dt);
        }
        if self.sixseven_enabled {
            self.sixseven.tick(dt);
        }
    }

    /// Consome o pedido de SixSeven deste tick.
    pub fn take_sixseven_request(&mut self) -> Option<SixSevenKey> {
        if self.sixseven_enabled {
            self.sixseven.take_request()
        } else {
            None
        }
    }

    pub fn speed_factor(self) -> f32 {
        if self.sixseven_enabled {
            self.sixseven.speed_factor()
        } else {
            1.0
        }
    }

    /// Volta ao estado de spawn sem perder o loadout (que vive em `Skills`).
    pub fn reset(&mut self) {
        let enabled = *self;
        *self = if enabled.walk_enabled && enabled.sixseven_enabled {
            Self::player()
        } else if enabled.walk_enabled {
            Self::enemy()
        } else {
            Self::none()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn player_has_walk_and_sixseven_but_enemy_has_only_walk() {
        assert!(Locomotion::player().has_walk());
        assert!(Locomotion::player().has_sixseven());
        assert!(Locomotion::enemy().has_walk());
        assert!(!Locomotion::enemy().has_sixseven());
        assert!(!Locomotion::none().has_walk());
    }

    #[test]
    fn sixseven_request_is_ignored_without_the_mechanic() {
        let mut enemy = Locomotion::enemy();
        let command = PlayerCommand {
            sixseven: Some(SixSevenKey::Six),
            ..PlayerCommand::none()
        };
        enemy.apply_command(command);
        assert!(enemy.take_sixseven_request().is_none());
    }
}
