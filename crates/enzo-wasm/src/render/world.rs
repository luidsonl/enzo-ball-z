//! Desenho do mundo: orquestração.
//!
//! Render de corpo/animação de personagens fica em [`super::entities`].
//! Render de projéteis e overlays de ataque fica em [`super::skills`].

use enzo_core::map::Level;
use enzo_core::snapshot::{EntitySnapshot, SnapshotKind, WorldSnapshot};
use macroquad::prelude::*;

use crate::camera::GameCamera;
use crate::convert::{to_rect, to_vec2};
use crate::effects::Effects;
use crate::sprites::Sprites;
use crate::tiles::TileRenderer;

use super::{entities, skills};

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
        Color::from_rgba(246, 196, 69, 22),
    );
    // Flag visual is rendered as 2 map blocks (flag_bottom + flag_top).
    let _ = sprites;
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
        entities::items::draw(sprites, entity, interpolate(previous, entity, alpha), time);
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
        entities::slime::draw(sprites, entity, interpolate(previous, entity, alpha), time);
    }
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
        skills::draw_projectile(sprites, entity, interpolate(previous, entity, alpha), time);
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
        let Some(player) = entity.player.as_ref() else {
            continue;
        };
        let pos = interpolate(previous, entity, alpha);
        match player.character {
            enzo_core::entities::PlayableCharacter::Valentina => {
                entities::valentina::draw(sprites, entity, player, pos, time);
            }
            _ => {
                entities::enzo::draw(sprites, entity, player, pos, time);
            }
        }
    }
}

fn draw_effects_world(effects: &Effects) {
    for particle in &effects.particles {
        let alpha = (particle.life / particle.max_life).clamp(0.0, 1.0);
        let color = Color::new(
            particle.color.r,
            particle.color.g,
            particle.color.b,
            particle.color.a * alpha,
        );
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
        let color = Color::new(
            text.color.r,
            text.color.g,
            text.color.b,
            text.color.a * alpha,
        );
        let shadow = Color::from_rgba(10, 8, 18, (alpha * 190.0) as u8);
        let dimensions = measure_text(&text.text, None, text.size as u16, 1.0);
        let x = text.pos.x - dimensions.width * 0.5;
        draw_text(&text.text, x + 1.0, text.pos.y + 1.0, text.size, shadow);
        draw_text(&text.text, x, text.pos.y, text.size, color);
    }
}

fn interpolate(previous: &WorldSnapshot, entity: &EntitySnapshot, alpha: f32) -> Vec2 {
    let Some(before) = previous.entities.iter().find(|item| item.id == entity.id) else {
        return to_vec2(entity.pos);
    };
    let delta = entity.pos - before.pos;
    if delta.length() > 220.0 {
        return to_vec2(entity.pos);
    }
    to_vec2(before.pos + delta * alpha.clamp(0.0, 1.0))
}
