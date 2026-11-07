//! Andar e correr por double-tap.
//!
//! O componente guarda apenas a intenção e o estado da corrida. Aceleração,
//! desaceleração e colisão continuam em [`crate::physics`].

use crate::entities::{AttributeSet, Body, EquipmentTags, Stats};
use crate::physics;
use crate::skills::{EntityType, Requirements};

pub const RUN_TAP_WINDOW: f32 = 0.25;

/// Andar é inerente a jogáveis e inimigos.
pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable, EntityType::Enemy],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NONE,
);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Walk {
    pub intent_x: i8,
    pub running: bool,
    pub last_move_x: i8,
    pub last_tap_dir: i8,
    pub tap_timer: f32,
}

impl Walk {
    pub const fn new() -> Self {
        Self {
            intent_x: 0,
            running: false,
            last_move_x: 0,
            last_tap_dir: 0,
            tap_timer: 0.0,
        }
    }

    pub fn apply(&mut self, move_x: i8) {
        self.intent_x = move_x.clamp(-1, 1);
    }

    pub fn tick(&mut self, dt: f32) {
        self.tap_timer = (self.tap_timer - dt).max(0.0);
        if self.tap_timer <= 0.0 {
            self.last_tap_dir = 0;
        }
    }

    pub fn update(&mut self, body: &mut Body, stats: &Stats, speed_factor: f32, dt: f32) {
        self.update_run_state();

        if self.intent_x != 0 {
            body.facing = self.intent_x;
        }

        let base = if self.running {
            stats.run_speed
        } else {
            stats.walk_speed
        };
        let target = self.intent_x as f32 * base * speed_factor;
        physics::horizontal_step(body, target, body.on_ground, dt);
    }

    /// Detecta duas pressões na mesma direção. Segurar mantém a corrida; soltar,
    /// parar ou inverter cancela.
    pub fn update_run_state(&mut self) {
        if self.intent_x == 0 {
            self.running = false;
            self.last_move_x = 0;
            return;
        }

        let previous = self.last_move_x;
        if previous != 0 && previous != self.intent_x {
            self.running = false;
        }

        if self.intent_x != previous {
            if self.last_tap_dir == self.intent_x && self.tap_timer > 0.0 {
                self.running = true;
                self.last_tap_dir = 0;
                self.tap_timer = 0.0;
            } else {
                self.last_tap_dir = self.intent_x;
                self.tap_timer = RUN_TAP_WINDOW;
            }
        }

        self.last_move_x = self.intent_x;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;

    fn body() -> Body {
        Body::new(Vec2::ZERO, crate::entities::ENZO_SIZE)
    }

    fn stats() -> Stats {
        Stats::player(crate::entities::Attributes::ZERO)
    }

    #[test]
    fn double_tap_starts_running() {
        let mut walk = Walk::default();
        for intent in [1, 0, 1] {
            walk.intent_x = intent;
            walk.update_run_state();
        }
        assert!(walk.running);
    }

    #[test]
    fn releasing_or_reversing_cancels_run() {
        let mut walk = Walk::default();
        for intent in [1, 0, 1] {
            walk.intent_x = intent;
            walk.update_run_state();
        }
        assert!(walk.running);

        walk.intent_x = 0;
        walk.update_run_state();
        assert!(!walk.running);

        for intent in [1, 0, 1] {
            walk.intent_x = intent;
            walk.update_run_state();
        }
        assert!(walk.running);
        walk.intent_x = -1;
        walk.update_run_state();
        assert!(!walk.running);
    }

    #[test]
    fn reaches_stats_speed() {
        let mut body = body();
        let mut walk = Walk::default();
        let stats = stats();
        walk.intent_x = 1;

        for _ in 0..120 {
            walk.update(&mut body, &stats, 1.0, crate::DT);
        }

        assert_eq!(body.vel.x, stats.walk_speed);
    }

    #[test]
    fn sixseven_speed_factor_slows_walk() {
        let mut body = body();
        let mut walk = Walk::default();
        let stats = stats();
        walk.intent_x = 1;

        for _ in 0..120 {
            walk.update(&mut body, &stats, 0.45, crate::DT);
        }

        assert!((body.vel.x - stats.walk_speed * 0.45).abs() < 0.01);
    }
}
