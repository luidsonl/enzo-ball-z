//! Visual do ataque corpo a corpo (modular skill assets).

use enzo_core::skills::melee::MELEE_ANIM_TIME;
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::sprites::{skill_fx, Sprites};

pub fn draw_overlay(
    sprites: &Sprites,
    entity: &EntitySnapshot,
    player: &PlayerSnapshot,
    center: Vec2,
) {
    if player.melee_timer <= 0.0 {
        return;
    }
    let progress = 1.0 - (player.melee_timer / MELEE_ANIM_TIME).clamp(0.0, 1.0);
    let _phase = if progress < 0.34 {
        0
    } else if progress < 0.67 {
        1
    } else {
        2
    };
    let facing = if entity.facing >= 0 { 1.0 } else { -1.0 };
    let alpha = (player.melee_timer / MELEE_ANIM_TIME).clamp(0.0, 1.0);
    sprites.draw_skill_fx(
        skill_fx::MELEE_IMPACT,
        Rect::new(center.x - 16.0 + facing * 14.0, center.y - 16.0, 32.0, 32.0),
        facing < 0.0,
        Color::from_rgba(255, 255, 255, (110.0 + 145.0 * alpha) as u8),
    );
}
