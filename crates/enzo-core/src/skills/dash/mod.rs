//! Rajada horizontal curta com cooldown.

use crate::entities::{AttributeSet, Body, EquipmentTags};

use super::{
    Activation, EntityType, Requirements, SkillAssets, SkillCategory, SkillDef, SkillId,
    SkillSource, SkillTiming,
};

pub const DASH_SPEED: f32 = 520.0;
pub const DASH_DURATION: f32 = 0.14;
pub const DASH_COOLDOWN: f32 = 0.70;

pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NONE,
);

pub static DEF: SkillDef = SkillDef {
    id: SkillId::Dash,
    category: SkillCategory::Movement,
    source: SkillSource::Innate,
    activation: Activation::Press,
    requirements: REQUIREMENTS,
    aura_cost: 0.0,
    cooldown: DASH_COOLDOWN,
    timing: SkillTiming::PrePhysics,
    assets: SkillAssets {
        icon: "assets/skills/dash/icon.png",
        entity: None,
        particles: Some("assets/skills/dash/trail.png"),
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    pub active: f32,
    pub pending: bool,
    pub direction: i8,
    pub intent_x: i8,
}

impl State {
    pub fn request(&mut self, facing: i8) {
        self.pending = true;
        self.direction = if facing >= 0 { 1 } else { -1 };
    }

    pub fn tick(&mut self, dt: f32) {
        self.active = (self.active - dt).max(0.0);
    }

    pub fn tick_timers(&mut self, _dt: f32) {}

    /// Tenta iniciar o dash. O cooldown central fica em `Skills.cooldowns`.
    pub fn try_start(&mut self, body: &Body, intent_x: i8) -> bool {
        if !self.pending || self.active > 0.0 {
            self.pending = false;
            return false;
        }
        self.pending = false;
        self.direction = if intent_x != 0 {
            intent_x.signum()
        } else if body.facing != 0 {
            body.facing.signum()
        } else {
            1
        };
        self.active = DASH_DURATION;
        true
    }

    pub fn is_active(&self) -> bool {
        self.active > 0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;

    #[test]
    fn dash_starts_and_expires() {
        let mut state = State::default();
        let mut body = Body::new(Vec2::ZERO, crate::entities::ENZO_SIZE);
        body.facing = 1;
        state.request(1);
        assert!(state.try_start(&body, 1));
        assert!(state.is_active());
        state.tick(DASH_DURATION);
        assert!(!state.is_active());
    }
}
