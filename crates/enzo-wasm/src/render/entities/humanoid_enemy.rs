//! Rendering for humanoid enemies/variants based on Enzo/Valentina rigs.
//!
//! Uses entity-level `visual_character` and `visual_profile` so enemies can have
//! varied appearances and the same data can be replicated in multiplayer.

use enzo_core::entities::PlayableCharacter;
use enzo_core::snapshot::EntitySnapshot;
use macroquad::prelude::*;

use crate::sprites::Sprites;

pub fn draw(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    let Some(character) = entity.visual_character else {
        return;
    };
    let size = vec2(entity.size.x, entity.size.y);
    let anim = entity.animation;

    // Shadow
    draw_ellipse(
        pos.x + size.x * 0.5,
        pos.y + size.y - 2.0,
        size.x * 0.475,
        3.5,
        0.0,
        Color::from_rgba(12, 10, 22, 70),
    );

    let blink = entity.iframes > 0.0 && (time * 24.0) as i32 % 2 == 0 && !anim.is_dead();
    let tint = if blink {
        Color::from_rgba(255, 255, 255, 90)
    } else if anim.is_dead() {
        Color::from_rgba(220, 220, 240, 180)
    } else {
        WHITE
    };

    let dest = Rect::new(
        pos.x + size.x * 0.5 - 16.0,
        pos.y + size.y - 32.0,
        32.0,
        32.0,
    );
    let flip = entity.facing < 0;
    let clip = clip_name(anim);
    let profile = entity.visual_profile;
    match character {
        PlayableCharacter::Valentina => {
            sprites.draw_valentina_rig(clip, time, dest, flip, tint, profile);
        }
        PlayableCharacter::Enzo => {
            sprites.draw_enzo_rig(clip, time, dest, flip, tint, profile);
        }
    }

    draw_health(entity, pos);
}

fn clip_name(anim: enzo_core::entities::AnimationState) -> &'static str {
    anim.clip_name()
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
