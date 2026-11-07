//! Desenho do mundo: tiles, entidades, projéteis e efeitos.

use enzo_core::map::Level;
use enzo_core::skills::melee::MELEE_ANIM_TIME;
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot, SnapshotKind, WorldSnapshot};
use macroquad::prelude::*;

use crate::camera::GameCamera;
use crate::convert::{to_rect, to_vec2};
use crate::effects::Effects;
use crate::sprites::{fx, Sprites};
use crate::tiles::TileRenderer;

use super::poses::{enzo_frame, item_frame, slime_frame, PlayerPose};

/// Tudo que o render do mundo precisa em um frame.
pub struct RenderContext<'a> {
    pub sprites: &'a Sprites,
    pub tiles: &'a mut TileRenderer,
    pub level: &'a Level,
    pub previous: &'a WorldSnapshot,
    pub current: &'a WorldSnapshot,
    pub alpha: f32,
    pub camera: &'a GameCamera,
    pub effects: &'a Effects,
    pub time: f32,
}

/// Desenha o mundo inteiro. Deve ser chamado depois de `draw_background`.
pub fn draw_world(ctx: &mut RenderContext) {
    let sprites = ctx.sprites;
    let previous = ctx.previous;
    let current = ctx.current;
    let alpha = ctx.alpha;
    let camera = ctx.camera;
    let time = ctx.time;

    set_camera(&camera.camera());

    let view = camera.view_rect();
    ctx.tiles.draw(ctx.level, view);
    draw_flag(sprites, current, time);
    draw_items(sprites, previous, current, alpha, time);
    draw_enemies(sprites, previous, current, alpha, time);
    draw_projectiles(sprites, previous, current, alpha, time);
    draw_players(sprites, previous, current, alpha, time);
    draw_effects_world(ctx.effects);

    set_default_camera();
}

fn draw_flag(sprites: &Sprites, snapshot: &WorldSnapshot, time: f32) {
    let Some(flag) = snapshot.flag else {
        return;
    };
    let flag = to_rect(flag);
    let pulse = 0.5 + 0.5 * (time * 3.0).sin();
    draw_circle(
        flag.x + 10.0,
        flag.y + 18.0,
        26.0 + pulse * 5.0,
        rgba(246, 196, 69, 22),
    );
    sprites.draw_flag(Rect::new(flag.x - 5.0, flag.y - 1.0, 32.0, 64.0), WHITE);
}

fn draw_items(
    sprites: &Sprites,
    previous: &WorldSnapshot,
    current: &WorldSnapshot,
    alpha: f32,
    time: f32,
) {
    for entity in &current.entities {
        if entity.kind != SnapshotKind::Item {
            continue;
        }
        let Some(data) = entity.item.as_ref() else {
            continue;
        };
        let pos = interpolate(previous, entity, alpha);
        let size = to_vec2(entity.size);
        let bob = (time * 3.0 + entity.id.0 as f32).sin() * 2.0;
        let center = pos + size * 0.5 + vec2(0.0, bob);
        draw_circle(center.x, center.y, 14.0, rgba(246, 196, 69, 30));
        sprites.draw_item(
            item_frame(data.kind),
            Rect::new(center.x - 8.0, center.y - 8.0, 16.0, 16.0),
            WHITE,
        );
    }
}

fn draw_enemies(
    sprites: &Sprites,
    previous: &WorldSnapshot,
    current: &WorldSnapshot,
    alpha: f32,
    time: f32,
) {
    for entity in &current.entities {
        if entity.kind != SnapshotKind::Enemy {
            continue;
        }
        let Some(data) = entity.enemy.as_ref() else {
            continue;
        };
        let pos = interpolate(previous, entity, alpha);
        let size = to_vec2(entity.size);
        let center = pos + size * 0.5;
        let frame = slime_frame(data.stun, time);
        let tint = if data.stun > 0.0 {
            rgba(255, 210, 210, 255)
        } else {
            WHITE
        };

        draw_shadow(pos, size);
        sprites.draw_slime(
            frame,
            Rect::new(center.x - 16.0, pos.y + size.y - 32.0, 32.0, 32.0),
            entity.facing < 0,
            tint,
        );
        draw_enemy_health(entity, pos);
    }
}

fn draw_enemy_health(entity: &EntitySnapshot, pos: Vec2) {
    if entity.health >= entity.max_health {
        return;
    }
    let size = to_vec2(entity.size);
    let width = size.x.max(20.0);
    let x = pos.x + size.x * 0.5 - width * 0.5;
    let y = pos.y - 8.0;
    let ratio = (entity.health / entity.max_health).clamp(0.0, 1.0);
    draw_rectangle(x - 1.0, y - 1.0, width + 2.0, 5.0, rgba(10, 8, 18, 210));
    draw_rectangle(x, y, width * ratio, 3.0, rgba(232, 72, 63, 255));
    draw_rectangle(x, y, width * ratio, 1.0, rgba(255, 128, 116, 255));
}

fn draw_projectiles(
    sprites: &Sprites,
    previous: &WorldSnapshot,
    current: &WorldSnapshot,
    alpha: f32,
    time: f32,
) {
    for entity in &current.entities {
        if entity.kind != SnapshotKind::Projectile {
            continue;
        }
        let pos = interpolate(previous, entity, alpha);
        let size = to_vec2(entity.size);
        let center = pos + size * 0.5;
        let direction = if entity.vel.x >= 0.0 { 1.0 } else { -1.0 };

        for index in 1..4 {
            let trail = center - vec2(direction * index as f32 * 8.0, 0.0);
            sprites.draw_fx(
                fx::MOGGING,
                Rect::new(trail.x - 16.0, trail.y - 16.0, 32.0, 32.0),
                direction < 0.0,
                rgba(255, 255, 255, 40 - index as u8 * 8),
            );
        }

        let pulse = 0.5 + 0.5 * (time * 20.0).sin();
        draw_circle(
            center.x,
            center.y,
            13.0 + pulse * 3.0,
            rgba(94, 242, 255, 28),
        );
        sprites.draw_fx(
            fx::MOGGING,
            Rect::new(center.x - 16.0, center.y - 16.0, 32.0, 32.0),
            direction < 0.0,
            WHITE,
        );
    }
}

fn draw_players(
    sprites: &Sprites,
    previous: &WorldSnapshot,
    current: &WorldSnapshot,
    alpha: f32,
    time: f32,
) {
    for entity in &current.entities {
        if entity.kind != SnapshotKind::Player {
            continue;
        }
        let Some(data) = entity.player.as_ref() else {
            continue;
        };
        let pos = interpolate(previous, entity, alpha);
        let size = to_vec2(entity.size);
        let center = pos + size * 0.5;
        let pose = PlayerPose {
            on_ground: entity.on_ground,
            vel_x: entity.vel.x,
            vel_y: entity.vel.y,
            dashing: entity.dashing,
            farming: data.farming,
            melee_timer: data.melee_timer,
            melee_step: data.melee_step,
            iframes: entity.iframes,
        };

        draw_shadow(pos, size);

        if data.farming {
            let pulse = 0.5 + 0.5 * (time * 8.0).sin();
            draw_circle(
                center.x,
                center.y,
                20.0 + pulse * 5.0,
                rgba(94, 242, 255, 22),
            );
        }

        let blink = entity.iframes > 0.0 && (time * 24.0) as i32 % 2 == 0;
        let tint = if blink {
            rgba(255, 255, 255, 90)
        } else {
            WHITE
        };
        sprites.draw_enzo(
            enzo_frame(pose, time),
            Rect::new(center.x - 16.0, pos.y + size.y - 32.0, 32.0, 32.0),
            entity.facing < 0,
            tint,
        );

        if data.melee_timer > 0.0 {
            draw_melee_slash(sprites, entity, center, data);
        }
    }
}

fn draw_melee_slash(
    sprites: &Sprites,
    entity: &EntitySnapshot,
    center: Vec2,
    data: &PlayerSnapshot,
) {
    let progress = 1.0 - (data.melee_timer / MELEE_ANIM_TIME).clamp(0.0, 1.0);
    let frame = if progress < 0.34 {
        fx::SLASH_A
    } else if progress < 0.67 {
        fx::SLASH_B
    } else {
        fx::SLASH_C
    };
    let facing = if entity.facing >= 0 { 1.0 } else { -1.0 };
    let alpha = (data.melee_timer / MELEE_ANIM_TIME).clamp(0.0, 1.0);
    sprites.draw_fx(
        frame,
        Rect::new(center.x - 16.0 + facing * 14.0, center.y - 16.0, 32.0, 32.0),
        facing < 0.0,
        rgba(255, 255, 255, (110.0 + 145.0 * alpha) as u8),
    );
}

fn draw_effects_world(effects: &Effects) {
    for particle in &effects.particles {
        let alpha = (particle.life / particle.max_life).clamp(0.0, 1.0);
        let color = with_alpha(particle.color, alpha);
        draw_rectangle(
            particle.pos.x - particle.size * 0.5,
            particle.pos.y - particle.size * 0.5,
            particle.size,
            particle.size,
            color,
        );
    }

    for text in &effects.texts {
        let alpha = (text.life / text.max_life).clamp(0.0, 1.0);
        let color = with_alpha(text.color, alpha);
        let shadow = rgba(10, 8, 18, (alpha * 190.0) as u8);
        let dimensions = measure_text(&text.text, None, text.size as u16, 1.0);
        let x = text.pos.x - dimensions.width * 0.5;
        draw_text(&text.text, x + 1.0, text.pos.y + 1.0, text.size, shadow);
        draw_text(&text.text, x, text.pos.y, text.size, color);
    }
}

fn draw_shadow(pos: Vec2, size: Vec2) {
    let width = size.x * 0.95;
    draw_ellipse(
        pos.x + size.x * 0.5,
        pos.y + size.y - 2.0,
        width * 0.5,
        3.5,
        0.0,
        rgba(12, 10, 22, 70),
    );
}

fn interpolate(previous: &WorldSnapshot, entity: &EntitySnapshot, alpha: f32) -> Vec2 {
    let Some(before) = previous.entities.iter().find(|item| item.id == entity.id) else {
        return to_vec2(entity.pos);
    };
    let delta = entity.pos - before.pos;
    // Evita deslizar pela tela em respawn/teleporte.
    if delta.length() > 220.0 {
        return to_vec2(entity.pos);
    }
    to_vec2(before.pos + delta * alpha.clamp(0.0, 1.0))
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    Color::new(color.r, color.g, color.b, color.a * alpha.clamp(0.0, 1.0))
}
