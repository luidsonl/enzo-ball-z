//! Pulo com coyote time, buffer e altura variável.

use crate::entities::{AttributeSet, Body, EquipmentTags, Stats};
use crate::input::PlayerCommand;
use crate::moves::{EntityType, Requirements};

pub const COYOTE_TIME: f32 = 0.10;
pub const JUMP_BUFFER_TIME: f32 = 0.12;

/// Pular é exclusivo de jogáveis no MVP.
pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NONE,
);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Jump {
    pub held: bool,
    pub coyote: f32,
    pub buffer: f32,
    pub cut_pending: bool,
}

impl Jump {
    pub fn apply(&mut self, command: PlayerCommand) {
        self.held = command.jump_held;
        if command.jump {
            self.buffer = JUMP_BUFFER_TIME;
        }
    }

    pub fn tick(&mut self, body: &Body, dt: f32) {
        if body.on_ground {
            self.coyote = COYOTE_TIME;
        } else {
            self.coyote = (self.coyote - dt).max(0.0);
        }
        self.buffer = (self.buffer - dt).max(0.0);
    }

    pub fn update(&mut self, body: &mut Body, stats: &Stats) {
        if self.buffer > 0.0 && self.coyote > 0.0 {
            body.vel.y = -stats.jump_speed;
            body.on_ground = false;
            self.buffer = 0.0;
            self.coyote = 0.0;
            self.cut_pending = true;
        }

        if body.vel.y >= 0.0 {
            self.cut_pending = false;
        } else if self.cut_pending && !self.held {
            body.vel.y *= 0.5;
            self.cut_pending = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map::Level;
    use crate::math::Vec2;

    fn level(source: &str) -> Level {
        Level::parse(source).unwrap()
    }

    fn command(jump: bool, held: bool) -> PlayerCommand {
        PlayerCommand {
            jump,
            jump_held: held,
            ..PlayerCommand::none()
        }
    }

    fn body_at(x: f32, y: f32) -> Body {
        Body::new(Vec2::new(x, y), crate::entities::PLAYER_SIZE)
    }

    #[test]
    fn jump_only_when_grounded_and_cuts_on_release() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 2.0);
        let mut jump = Jump::default();
        let stats = Stats::player(crate::entities::Attributes::ZERO);

        // Primeiro tick pousa no chão.
        jump.tick(&body, crate::DT);
        crate::physics::apply_gravity(&mut body, crate::DT);
        crate::physics::integrate(&mut body, &level, crate::DT);
        assert!(body.on_ground);

        jump.apply(command(true, true));
        jump.tick(&body, crate::DT);
        jump.update(&mut body, &stats);
        assert!(body.vel.y < 0.0);
        let rising = body.vel.y;

        jump.apply(command(false, false));
        jump.update(&mut body, &stats);
        assert!(body.vel.y > rising);
    }

    #[test]
    fn coyote_time_allows_jump_after_leaving_ledge() {
        let level = level("P...\n####\n");
        let mut body = body_at(0.0, 2.0);
        let mut jump = Jump::default();
        let stats = Stats::player(crate::entities::Attributes::ZERO);

        jump.tick(&body, crate::DT);
        crate::physics::apply_gravity(&mut body, crate::DT);
        crate::physics::integrate(&mut body, &level, crate::DT);

        // Sai da borda: fica sem chão, mas ainda dentro do coyote.
        body.pos.x = 200.0;
        jump.apply(command(true, true));
        jump.tick(&body, crate::DT);
        jump.update(&mut body, &stats);
        assert!(body.vel.y < 0.0);
    }
}
