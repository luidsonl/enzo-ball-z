//! Impulso horizontal curto com cooldown.

use crate::entity::{Attributes, Body};
use crate::moves::{EntityType, Requirements};

pub const DASH_SPEED: f32 = 520.0;
pub const DASH_DURATION: f32 = 0.14;
pub const DASH_COOLDOWN: f32 = 0.70;

/// Dash é exclusivo de jogadores no MVP.
pub const REQUIREMENTS: Requirements =
    Requirements::new(&[EntityType::Player], 1, Attributes::ZERO);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dash {
    pub pending: bool,
    pub direction: i8,
    pub timer: f32,
    pub cooldown: f32,
}

impl Default for Dash {
    fn default() -> Self {
        Self {
            pending: false,
            direction: 1,
            timer: 0.0,
            cooldown: 0.0,
        }
    }
}

impl Dash {
    pub fn request(&mut self) {
        self.pending = true;
    }

    pub fn tick(&mut self, dt: f32) {
        self.cooldown = (self.cooldown - dt).max(0.0);
        self.timer = (self.timer - dt).max(0.0);
    }

    pub fn is_active(self) -> bool {
        self.timer > 0.0
    }

    /// Consome o pedido. Sem direção segurada, usa o facing atual.
    pub fn try_start(&mut self, body: &mut Body, intent_x: i8) {
        if !self.pending {
            return;
        }
        self.pending = false;
        if self.cooldown > 0.0 || self.timer > 0.0 {
            return;
        }

        let direction = if intent_x != 0 { intent_x } else { body.facing };
        self.direction = direction;
        self.timer = DASH_DURATION;
        self.cooldown = DASH_COOLDOWN;
        body.facing = direction;
    }
}
