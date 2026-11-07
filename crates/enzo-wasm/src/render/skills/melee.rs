//! Visual do ataque corpo a corpo (slash).

use enzo_core::skills::melee::MELEE_ANIM_TIME;
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::sprites::{self, Sprites};

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
    let frame = if progress < 0.34 {
        sprites::fx::SLASH_A
    } else if progress < 0.67 {
        sprites::fx::SLASH_B
    } else {
        sprites::fx::SLASH_C
    };
    let facing = if entity.facing >= 0 { 1.0 } else { -1.0 };
    let alpha = (player.melee_timer / MELEE_ANIM_TIME).clamp(0.0, 1.0);
    sprites.draw_fx(
        frame,
        Rect::new(center.x - 16.0 + facing * 14.0, center.y - 16.0, 32.0, 32.0),
        facing < 0.0,
        Color::from_rgba(255, 255, 255, (110.0 + 145.0 * alpha) as u8),
    );
}
