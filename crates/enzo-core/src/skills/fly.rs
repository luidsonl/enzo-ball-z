//! Voo controlado. Desabilita a gravidade enquanto equipado/ativo.

use crate::entities::{AttributeId, AttributeSet, Body, EquipmentTags};
use crate::math::approach;
use crate::physics;

use super::{
    Activation, EntityType, Requirements, SkillAssets, SkillCategory, SkillDef, SkillId,
    SkillSource, SkillTiming,
};

pub const FLY_SPEED: f32 = 180.0;
pub const FLY_ACCEL: f32 = 900.0;

pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    5,
    AttributeSet::ZERO.with(AttributeId::AURA, 5),
    EquipmentTags::NONE,
);

pub static DEF: SkillDef = SkillDef {
    id: SkillId::Fly,
    category: SkillCategory::Movement,
    source: SkillSource::Progression,
    activation: Activation::Hold,
    requirements: REQUIREMENTS,
    aura_cost: 0.0,
    cooldown: 0.0,
    timing: SkillTiming::PrePhysics,
    assets: SkillAssets {
        icon: "assets/skills/fly/icon.png",
        entity: None,
        particles: Some("assets/skills/fly/aura.png"),
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    pub vertical_intent: i8,
    pub active: bool,
}

impl State {
    pub fn update(&mut self, body: &mut Body, dt: f32) {
        let target = self.vertical_intent as f32 * FLY_SPEED;
        body.vel.y = approach(body.vel.y, target, FLY_ACCEL * dt);
        if self.vertical_intent == 0 && body.vel.y.abs() < 5.0 {
            body.vel.y = 0.0;
        }
        // Se não estiver ativo (intent 0 mas skill equipada), gravidade normal.
        if self.vertical_intent == 0 {
            // Mantém leve inércia? O antigo fly sempre sobrescrevia gravidade quando equipado.
            // Comportamento preservado: fly equipado = sem gravidade enquanto `update` roda no ramo fly.
            body.vel.y = approach(body.vel.y, 0.0, FLY_ACCEL * dt * 0.5);
        }
        body.on_ground = false;
        let _ = physics::GRAVITY; // referência explícita ao contrato antigo
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;

    #[test]
    fn fly_moves_vertically() {
        let mut state = State {
            vertical_intent: -1,
            active: true,
        };
        let mut body = Body::new(Vec2::ZERO, crate::entities::ENZO_SIZE);
        for _ in 0..30 {
            state.update(&mut body, 1.0 / 60.0);
        }
        assert!(body.vel.y < 0.0);
    }
}
