//! Jump with coyote time, jump buffer, and variable height.

use crate::entities::{AttributeSet, Body, EquipmentTags, Stats};

use super::{
    Activation, EntityType, Requirements, SkillAssets, SkillCategory, SkillDef, SkillId,
    SkillSource, SkillTiming,
};

pub const COYOTE_TIME: f32 = 0.10;
pub const JUMP_BUFFER_TIME: f32 = 0.12;

pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NONE,
);

pub static DEF: SkillDef = SkillDef {
    id: SkillId::Jump,
    category: SkillCategory::Movement,
    source: SkillSource::Innate,
    activation: Activation::PressHold,
    requirements: REQUIREMENTS,
    aura_cost: 0.0,
    cooldown: 0.0,
    timing: SkillTiming::PrePhysics,
    assets: SkillAssets {
        icon: "assets/skills/jump/icon.png",
        entity: None,
        particles: Some("assets/skills/jump/dust.png"),
    },
};

/// Visual/animation metadata co-located with the skill module.
pub mod visual {
    pub const ICON: &str = "assets/skills/jump/icon.png";
    pub const DUST: &str = "assets/skills/jump/dust.png";
    pub const DUST_DURATION: f32 = 0.20;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    pub held: bool,
    pub coyote: f32,
    pub buffer: f32,
    pub cut_pending: bool,
}

impl State {
    pub fn tick(&mut self, body: &Body, dt: f32) {
        if body.on_ground {
            self.coyote = COYOTE_TIME;
        } else {
            self.coyote = (self.coyote - dt).max(0.0);
        }
        self.buffer = (self.buffer - dt).max(0.0);
    }

    pub fn tick_timers(&mut self, _dt: f32) {}

    pub fn update(&mut self, body: &mut Body, stats: &Stats) {
        if self.buffer > 0.0 && self.coyote > 0.0 {
            body.vel.y = -stats.jump_speed;
            body.on_ground = false;
            self.buffer = 0.0;
            self.coyote = 0.0;
            self.cut_pending = true;
        }

        if self.cut_pending && !self.held && body.vel.y < 0.0 {
            body.vel.y *= 0.5;
            self.cut_pending = false;
        }

        if body.vel.y >= 0.0 {
            self.cut_pending = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::Attributes;
    use crate::math::Vec2;

    fn body() -> Body {
        let mut body = Body::new(Vec2::ZERO, crate::entities::ENZO_SIZE);
        body.on_ground = true;
        body
    }

    #[test]
    fn jump_cuts_height_when_released() {
        let mut state = State::default();
        let mut body = body();
        let stats = Stats::player(Attributes::ZERO);
        state.buffer = JUMP_BUFFER_TIME;
        state.held = true;
        state.tick(&body, 0.0);
        state.update(&mut body, &stats);
        assert!(body.vel.y < 0.0);
        state.held = false;
        body.vel.y = -200.0;
        state.update(&mut body, &stats);
        assert_eq!(body.vel.y, -100.0);
    }
}
