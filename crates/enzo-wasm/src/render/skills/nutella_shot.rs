//! Visual do projétil Tiro de Nutela (modular skill assets).

use enzo_core::snapshot::EntitySnapshot;
use macroquad::prelude::*;

use crate::sprites::{skill_fx, Sprites};

pub fn draw_projectile(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    let size = vec2(entity.size.x, entity.size.y);
    let center = pos + size * 0.5;
    let _direction = if entity.vel.x >= 0.0 { 1.0 } else { -1.0 };
    let pulse = 0.5 + 0.5 * (time * 18.0).sin();
    let tint = Color::from_rgba(255, 245, 235, (200.0 + pulse * 55.0) as u8);
    sprites.draw_skill_fx(
        skill_fx::NUTELLA_PROJECTILE,
        Rect::new(center.x - 12.0, center.y - 12.0, 24.0, 24.0),
        entity.vel.x < 0.0,
        tint,
    );
}
