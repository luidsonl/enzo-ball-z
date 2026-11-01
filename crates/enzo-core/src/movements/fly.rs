//! Voo livre.
//!
//! O componente existe para inimigos voadores e itens futuros. No MVP o jogador
//! não nasce com `Fly`; ele só é atribuído quando os requisitos são cumpridos.

use crate::entities::{AttributeId, AttributeSet, Body, EquipmentTags};
use crate::math::approach;
use crate::moves::{EntityType, Requirements};

pub const FLY_SPEED: f32 = 180.0;
pub const FLY_ACCEL: f32 = 900.0;

/// Voo exige nível 5 e 5 pontos em Aura.
pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    5,
    AttributeSet::ZERO.with(AttributeId::AURA, 5),
    EquipmentTags::NONE,
);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Fly {
    pub vertical_intent: i8,
}

impl Fly {
    pub fn update(&mut self, body: &mut Body, dt: f32) {
        let target = self.vertical_intent as f32 * FLY_SPEED;
        body.vel.y = approach(body.vel.y, target, FLY_ACCEL * dt);
        body.on_ground = false;
    }
}
