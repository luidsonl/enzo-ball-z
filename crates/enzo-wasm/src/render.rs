//! Render do mundo a partir do snapshot.
//!
//! O render nunca muta o mundo. Posições são interpoladas entre o snapshot
//! anterior e o atual usando `alpha`; estados (pulo, dash, combo, i-frames)
//! vêm do snapshot atual.

use enzo_core::attacks::melee::MELEE_ANIM_TIME;
use enzo_core::entities::ItemKind;
use enzo_core::map::{Level, Tile, TILE_SIZE};
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot, SnapshotKind, WorldSnapshot};
use macroquad::prelude::*;

use crate::camera::GameCamera;
use crate::convert::{to_rect, to_vec2};
use crate::effects::Effects;
use crate::sprites::{enzo, fx, item, slime, tile, Sprites};

const CLOUDS: &[(f32, f32, f32, f32)] = &[
    (0.05, 0.15, 0.10, 1.1),
    (0.28, 0.24, 0.16, 0.8),
    (0.52, 0.12, 0.08, 1.3),
    (0.74, 0.27, 0.20, 0.9),
    (0.92, 0.18, 0.13, 1.0),
];

/// Estado visual do jogador. Isolado do snapshot para ser testável.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerPose {
    pub on_ground: bool,
    pub vel_x: f32,
    pub vel_y: f32,
    pub dashing: bool,
    pub farming: bool,
    pub melee_timer: f32,
    pub melee_step: u8,
    pub iframes: f32,
}

/// Escolhe o frame do Enzo. A ordem das prioridades define o que o jogador vê
/// quando dois estados coincidem (ataque > dano > dash > farming > ar > chão).
pub fn enzo_frame(pose: PlayerPose, time: f32) -> usize {
    if pose.melee_timer > 0.0 {
        return if pose.melee_step >= 3 {
            enzo::ATTACK_B
        } else {
            enzo::ATTACK_A
        };
    }
    if pose.iframes > 0.0 {
        return enzo::HURT;
    }
    if pose.dashing {
        return enzo::DASH;
    }
    if pose.farming {
        return if ((time * 10.0) as usize).is_multiple_of(2) {
            enzo::FARM_A
        } else {
            enzo::FARM_B
        };
    }
    if !pose.on_ground {
        return if pose.vel_y < 0.0 {
            enzo::JUMP
        } else {
            enzo::FALL
        };
    }
    if pose.vel_x.abs() > 20.0 {
        let phase = (time * (4.0 + pose.vel_x.abs() / 80.0)) as usize % 4;
        return [enzo::RUN_A, enzo::RUN_B, enzo::RUN_C, enzo::RUN_D][phase];
    }
    if ((time * 2.5) as usize).is_multiple_of(2) {
        enzo::IDLE_A
    } else {
        enzo::IDLE_B
    }
}

pub fn slime_frame(stun: f32, time: f32) -> usize {
    if stun > 0.0 {
        return slime::HURT;
    }
    match (time * 7.0) as usize % 3 {
        0 => slime::WALK_A,
        1 => slime::WALK_B,
        _ => slime::STRETCH,
    }
}

pub fn item_frame(kind: ItemKind) -> usize {
    match kind {
        ItemKind::Sword => item::SWORD,
        ItemKind::Bow => item::BOW,
        ItemKind::Armor => item::ARMOR,
        ItemKind::Amulet => item::AMULET,
        ItemKind::Potion => item::POTION,
    }
}

/// Fundo em screen space: gradiente, sol, nuvens e montanhas com parallax.
pub fn draw_background(camera: &GameCamera, time: f32) {
    let width = screen_width();
    let height = screen_height();

    let top = rgba(34, 46, 86, 255);
    let bottom = rgba(132, 176, 198, 255);
    let bands = 48;
    for index in 0..bands {
        let t = index as f32 / (bands - 1) as f32;
        let color = mix(top, bottom, t);
        draw_rectangle(
            0.0,
            index as f32 * height / bands as f32,
            width,
            height / bands as f32 + 1.0,
            color,
        );
    }

    // Sol com halo.
    let sun = vec2(width * 0.78, height * 0.17);
    draw_circle(sun.x, sun.y, height * 0.085, rgba(255, 236, 170, 26));
    draw_circle(sun.x, sun.y, height * 0.055, rgba(255, 240, 190, 52));
    draw_circle(sun.x, sun.y, height * 0.032, rgba(255, 248, 214, 150));

    // Nuvens com parallax horizontal.
    let span = width + 320.0;
    for (index, cloud) in CLOUDS.iter().enumerate() {
        let drift = time * (4.0 + index as f32 * 1.5) * cloud.3;
        let x = (cloud.0 * span - camera.center.x * cloud.2 + drift).rem_euclid(span) - 160.0;
        let y = cloud.1 * height;
        let s = cloud.3;
        let shade = if index % 2 == 0 {
            rgba(232, 236, 255, 210)
        } else {
            rgba(214, 220, 248, 190)
        };
        draw_ellipse(x, y, 44.0 * s, 15.0 * s, 0.0, shade);
        draw_ellipse(x - 26.0 * s, y + 5.0 * s, 24.0 * s, 10.0 * s, 0.0, shade);
        draw_ellipse(x + 28.0 * s, y + 6.0 * s, 26.0 * s, 11.0 * s, 0.0, shade);
        draw_ellipse(x + 4.0 * s, y - 8.0 * s, 24.0 * s, 12.0 * s, 0.0, shade);
    }

    // Montanhas distantes.
    let base = height * 0.88;
    let far = camera.center.x * 0.18;
    for index in 0..6 {
        let x = (index as f32 * width * 0.28 - far).rem_euclid(width + 500.0) - 250.0;
        draw_triangle(
            vec2(x, base),
            vec2(x + 360.0, base),
            vec2(x + 180.0, base - height * 0.30),
            rgba(74, 104, 126, 255),
        );
        draw_triangle(
            vec2(x + 120.0, base),
            vec2(x + 300.0, base),
            vec2(x + 210.0, base - height * 0.18),
            rgba(92, 126, 144, 255),
        );
    }

    // Faixa de neblina na base.
    draw_rectangle(
        0.0,
        height * 0.84,
        width,
        height * 0.16,
        rgba(180, 206, 214, 36),
    );
}

/// Tudo que o render do mundo precisa em um frame.
pub struct RenderContext<'a> {
    pub sprites: &'a Sprites,
    pub level: &'a Level,
    pub previous: &'a WorldSnapshot,
    pub current: &'a WorldSnapshot,
    pub alpha: f32,
    pub camera: &'a GameCamera,
    pub effects: &'a Effects,
    pub time: f32,
}

/// Desenha o mundo inteiro. Deve ser chamado depois de `draw_background`.
pub fn draw_world(ctx: &RenderContext) {
    let sprites = ctx.sprites;
    let previous = ctx.previous;
    let current = ctx.current;
    let alpha = ctx.alpha;
    let camera = ctx.camera;
    let time = ctx.time;

    set_camera(&camera.camera());

    draw_tiles(sprites, ctx.level, camera.view_rect());
    draw_flag(sprites, current, time);
    draw_items(sprites, previous, current, alpha, time);
    draw_enemies(sprites, previous, current, alpha, time);
    draw_projectiles(sprites, previous, current, alpha, time);
    draw_players(sprites, previous, current, alpha, time);
    draw_effects_world(ctx.effects);

    set_default_camera();
}

fn draw_tiles(sprites: &Sprites, level: &Level, view: Rect) {
    let min_x = ((view.x / TILE_SIZE).floor() as i32 - 1).max(0);
    let max_x = (((view.x + view.w) / TILE_SIZE).ceil() as i32 + 1).min(level.width() as i32);
    let min_y = ((view.y / TILE_SIZE).floor() as i32 - 1).max(0);
    let max_y = (((view.y + view.h) / TILE_SIZE).ceil() as i32 + 1).min(level.height() as i32);

    for ty in min_y..max_y {
        for tx in min_x..max_x {
            let tile = level.tile_at(tx, ty);
            let index = match tile {
                Tile::Empty => continue,
                Tile::Ground => {
                    let above = level.tile_at(tx, ty - 1);
                    if above.is_solid() {
                        tile::GROUND_FILL
                    } else {
                        tile::GROUND_TOP
                    }
                }
                Tile::Block => tile::BLOCK,
                Tile::Platform => tile::PLATFORM,
            };
            sprites.draw_tile(
                index,
                Rect::new(
                    tx as f32 * TILE_SIZE,
                    ty as f32 * TILE_SIZE,
                    TILE_SIZE,
                    TILE_SIZE,
                ),
                WHITE,
            );
        }
    }
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
        let SnapshotKind::Item(data) = &entity.kind else {
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
        let SnapshotKind::Enemy(data) = &entity.kind else {
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
        if !matches!(entity.kind, SnapshotKind::Projectile(_)) {
            continue;
        }
        let pos = interpolate(previous, entity, alpha);
        let size = to_vec2(entity.size);
        let center = pos + size * 0.5;
        let direction = if entity.vel.x >= 0.0 { 1.0 } else { -1.0 };

        for index in 1..4 {
            let trail = center - vec2(direction * index as f32 * 8.0, 0.0);
            sprites.draw_fx(
                fx::MOGAR,
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
            fx::MOGAR,
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
        let SnapshotKind::Player(data) = &entity.kind else {
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

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose() -> PlayerPose {
        PlayerPose {
            on_ground: true,
            vel_x: 0.0,
            vel_y: 0.0,
            dashing: false,
            farming: false,
            melee_timer: 0.0,
            melee_step: 0,
            iframes: 0.0,
        }
    }

    #[test]
    fn idle_and_run_frames_are_selected_by_state() {
        assert_eq!(enzo_frame(pose(), 0.0), enzo::IDLE_A);
        assert_eq!(enzo_frame(pose(), 0.5), enzo::IDLE_B);

        let running = PlayerPose {
            vel_x: 220.0,
            ..pose()
        };
        assert!([enzo::RUN_A, enzo::RUN_B, enzo::RUN_C, enzo::RUN_D]
            .contains(&enzo_frame(running, 0.0)));
    }

    #[test]
    fn air_dash_farm_and_attack_have_priority() {
        let jumping = PlayerPose {
            on_ground: false,
            vel_y: -100.0,
            ..pose()
        };
        assert_eq!(enzo_frame(jumping, 0.0), enzo::JUMP);

        let falling = PlayerPose {
            on_ground: false,
            vel_y: 100.0,
            ..pose()
        };
        assert_eq!(enzo_frame(falling, 0.0), enzo::FALL);

        let dashing = PlayerPose {
            dashing: true,
            ..pose()
        };
        assert_eq!(enzo_frame(dashing, 0.0), enzo::DASH);

        let farming = PlayerPose {
            farming: true,
            ..pose()
        };
        assert_eq!(enzo_frame(farming, 0.0), enzo::FARM_A);

        let attacking = PlayerPose {
            melee_timer: 0.1,
            melee_step: 3,
            ..pose()
        };
        assert_eq!(enzo_frame(attacking, 0.0), enzo::ATTACK_B);
    }

    #[test]
    fn hurt_frame_wins_over_movement() {
        let hurt = PlayerPose {
            iframes: 0.5,
            vel_x: 220.0,
            ..pose()
        };
        assert_eq!(enzo_frame(hurt, 0.0), enzo::HURT);
    }

    #[test]
    fn slime_uses_hurt_frame_when_stunned() {
        assert_eq!(slime_frame(0.0, 0.0), slime::WALK_A);
        assert_eq!(slime_frame(0.2, 0.0), slime::HURT);
    }

    #[test]
    fn every_item_has_a_frame() {
        assert_eq!(item_frame(ItemKind::Sword), item::SWORD);
        assert_eq!(item_frame(ItemKind::Bow), item::BOW);
        assert_eq!(item_frame(ItemKind::Armor), item::ARMOR);
        assert_eq!(item_frame(ItemKind::Amulet), item::AMULET);
        assert_eq!(item_frame(ItemKind::Potion), item::POTION);
    }
}
