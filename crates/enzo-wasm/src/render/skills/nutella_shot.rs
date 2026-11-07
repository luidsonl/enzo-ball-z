//! Visual do projétil Tiro de Nutela (placeholder).

use enzo_core::snapshot::EntitySnapshot;
use macroquad::prelude::*;

use crate::sprites::Sprites;

use super::mogging;

pub fn draw_projectile(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    // Placeholder: reutiliza base do projétil com tom diferente.
    let size = vec2(entity.size.x, entity.size.y);
    let center = pos + size * 0.5;
    let direction = if entity.vel.x >= 0.0 { 1.0 } else { -1.0 };
    let pulse = 0.5 + 0.5 * (time * 18.0).sin();
    draw_circle(
        center.x,
        center.y,
        8.0 + pulse * 2.0,
        Color::from_rgba(130, 80, 40, 220),
    );
    draw_circle(
        center.x - direction * 3.0,
        center.y,
        5.0,
        Color::from_rgba(90, 55, 25, 220),
    );
    let _ = (sprites, entity, mogging::FX_FRAME);
}
