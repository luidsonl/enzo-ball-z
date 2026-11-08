//! Animação e desenho da Slime (lazy modular entity sheet).

use enzo_core::snapshot::EntitySnapshot;
use macroquad::prelude::*;

use crate::sprites::{self, Sprites};

pub fn frame(stun: f32, time: f32) -> usize {
    use sprites::slime as f;
    if stun > 0.0 {
        return f::HURT;
    }
    match (time * 7.0) as usize % 3 {
        0 => f::WALK_A,
        1 => f::WALK_B,
        _ => f::STRETCH,
    }
}

pub fn draw(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    let Some(data) = entity.enemy.as_ref() else {
        return;
    };
    let size = vec2(entity.size.x, entity.size.y);
    let center = pos + size * 0.5;
    let frame = frame(data.stun, time);
    let tint = if data.stun > 0.0 {
        Color::from_rgba(255, 210, 210, 255)
    } else {
        WHITE
    };

    draw_ellipse(
        pos.x + size.x * 0.5,
        pos.y + size.y - 2.0,
        size.x * 0.475,
        3.5,
        0.0,
        Color::from_rgba(12, 10, 22, 70),
    );
    sprites.draw_slime(
        frame,
        Rect::new(center.x - 16.0, pos.y + size.y - 32.0, 32.0, 32.0),
        entity.facing < 0,
        tint,
    );
    draw_health(entity, pos);
}

fn draw_health(entity: &EntitySnapshot, pos: Vec2) {
    if entity.health >= entity.max_health {
        return;
    }
    let size = vec2(entity.size.x, entity.size.y);
    let width = size.x.max(20.0);
    let x = pos.x + size.x * 0.5 - width * 0.5;
    let y = pos.y - 8.0;
    let ratio = (entity.health / entity.max_health).clamp(0.0, 1.0);
    draw_rectangle(
        x - 1.0,
        y - 1.0,
        width + 2.0,
        5.0,
        Color::from_rgba(10, 8, 18, 210),
    );
    draw_rectangle(x, y, width * ratio, 3.0, Color::from_rgba(232, 72, 63, 255));
    draw_rectangle(
        x,
        y,
        width * ratio,
        1.0,
        Color::from_rgba(255, 128, 116, 255),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slime_uses_hurt_frame_when_stunned() {
        assert_eq!(frame(0.0, 0.0), sprites::slime::WALK_A);
        assert_eq!(frame(0.2, 0.0), sprites::slime::HURT);
    }
}
