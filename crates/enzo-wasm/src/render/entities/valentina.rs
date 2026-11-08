//! Animation and drawing for the playable Valentina character (rig/layer composition).

use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::sprites::Sprites;

use super::super::skills;
use super::compositor;

pub fn draw(
    sprites: &Sprites,
    entity: &EntitySnapshot,
    player: &PlayerSnapshot,
    pos: Vec2,
    time: f32,
) {
    let size = vec2(entity.size.x, entity.size.y);
    let center = pos + size * 0.5;
    let anim = entity.animation;

    draw_ellipse(
        pos.x + size.x * 0.5,
        pos.y + size.y - 2.0,
        size.x * 0.475,
        3.5,
        0.0,
        Color::from_rgba(12, 10, 22, 70),
    );

    if anim.farming || anim.is_sixseven() {
        let pulse = 0.5 + 0.5 * (time * 8.0).sin();
        draw_circle(
            center.x,
            center.y,
            20.0 + pulse * 5.0,
            Color::from_rgba(255, 150, 220, 22),
        );
    }

    let blink = entity.iframes > 0.0 && (time * 24.0) as i32 % 2 == 0 && !anim.is_dead();
    let tint = if blink {
        Color::from_rgba(255, 255, 255, 90)
    } else if anim.is_dead() {
        Color::from_rgba(220, 220, 240, 180)
    } else {
        WHITE
    };

    let dest = Rect::new(center.x - 16.0, pos.y + size.y - 32.0, 32.0, 32.0);
    let flip = entity.facing < 0;
    let clip = anim.clip_name();
    let source = sprites.valentina_rig.frame_rect(clip, time);
    sprites.draw_valentina_rig(clip, time, dest, flip, tint, player.visual_profile);
    compositor::draw_equipped_layers_rig(sprites, player, source, dest, flip, tint);

    skills::draw_player_overlays(sprites, entity, player, center);
}
