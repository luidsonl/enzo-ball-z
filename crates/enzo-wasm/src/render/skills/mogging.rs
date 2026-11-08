//! Visual do projétil/efeito Mogging (modular skill assets).

use enzo_core::snapshot::EntitySnapshot;
use macroquad::prelude::*;

use crate::sprites::{skill_fx, Sprites};

pub fn draw_projectile(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    let size = vec2(entity.size.x, entity.size.y);
    let center = pos + size * 0.5;
    let direction = if entity.vel.x >= 0.0 { 1.0 } else { -1.0 };

    for index in 1..4 {
        let trail = center - vec2(direction * index as f32 * 8.0, 0.0);
        sprites.draw_skill_fx(
            skill_fx::MOGGING_PROJECTILE,
            Rect::new(trail.x - 16.0, trail.y - 16.0, 32.0, 32.0),
            direction < 0.0,
            Color::from_rgba(255, 255, 255, 40_u8.saturating_sub(index as u8 * 8)),
        );
    }

    let pulse = 0.5 + 0.5 * (time * 20.0).sin();
    draw_circle(
        center.x,
        center.y,
        13.0 + pulse * 3.0,
        Color::from_rgba(94, 242, 255, 28),
    );
    sprites.draw_skill_fx(
        skill_fx::MOGGING_PROJECTILE,
        Rect::new(center.x - 16.0, center.y - 16.0, 32.0, 32.0),
        direction < 0.0,
        WHITE,
    );
}
