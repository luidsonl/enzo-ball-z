//! Gravidade, integração e colisão AABB com o tilemap.
//!
//! A colisão é resolvida em X e depois em Y. Plataformas (`=`) são
//! atravessáveis por baixo e por cima: só param o corpo quando ele está caindo
//! e vinha acima da superfície.

use crate::entity::{Body, PlayerData};
use crate::map::{Level, TILE_SIZE};
use crate::math::{approach, Rect};

pub const GRAVITY: f32 = 1800.0;
pub const MAX_FALL_SPEED: f32 = 900.0;
pub const GROUND_ACCEL: f32 = 1800.0;
pub const GROUND_DECEL: f32 = 2200.0;
pub const AIR_ACCEL: f32 = 1200.0;
pub const COYOTE_TIME: f32 = 0.10;
pub const JUMP_BUFFER_TIME: f32 = 0.12;
pub const RUN_TAP_WINDOW: f32 = 0.25;

/// Margem para evitar que o corpo fique exatamente sobre a borda do tile e
/// seja considerado sobreposto no tick seguinte.
const SKIN: f32 = 0.001;

/// Atualiza o estado de corrida a partir do double-tap.
///
/// O cliente só envia `move_x`; a detecção de duas pressões na mesma direção
/// (com soltura no meio) é regra de jogo e vive aqui.
pub fn update_run_state(player: &mut PlayerData, move_x: i8, dt: f32) {
    player.tap_timer = (player.tap_timer - dt).max(0.0);
    if player.tap_timer <= 0.0 {
        player.last_tap_dir = 0;
    }

    if move_x == 0 {
        player.running = false;
        player.last_move_x = 0;
        return;
    }

    let previous = player.last_move_x;
    if previous != 0 && previous != move_x {
        player.running = false;
    }

    if move_x != previous {
        if player.last_tap_dir == move_x && player.tap_timer > 0.0 {
            player.running = true;
            player.last_tap_dir = 0;
            player.tap_timer = 0.0;
        } else {
            player.last_tap_dir = move_x;
            player.tap_timer = RUN_TAP_WINDOW;
        }
    }

    player.last_move_x = move_x;
}

/// Acelera a velocidade horizontal em direção à intenção do jogador.
pub fn horizontal_step(body: &mut Body, target_speed: f32, on_ground: bool, dt: f32) {
    let accel = if on_ground {
        if target_speed == 0.0 {
            GROUND_DECEL
        } else {
            GROUND_ACCEL
        }
    } else if target_speed != 0.0 {
        AIR_ACCEL
    } else {
        // No ar sem input, mantém a inércia.
        return;
    };

    body.vel.x = approach(body.vel.x, target_speed, accel * dt);
}

pub fn apply_gravity(body: &mut Body, dt: f32) {
    body.vel.y = (body.vel.y + GRAVITY * dt).min(MAX_FALL_SPEED);
}

pub fn jump(body: &mut Body, jump_speed: f32) {
    body.vel.y = -jump_speed;
    body.on_ground = false;
}

/// Soltar o botão cedo corta a subida pela metade. Só vale uma vez por pulo.
pub fn cut_jump(body: &mut Body, player: &mut PlayerData) {
    if player.jump_cut && body.vel.y < 0.0 {
        body.vel.y *= 0.5;
        player.jump_cut = false;
    }
}

pub fn update_timers(player: &mut PlayerData, on_ground: bool, dt: f32) {
    if on_ground {
        player.coyote = COYOTE_TIME;
    } else {
        player.coyote = (player.coyote - dt).max(0.0);
    }
    player.jump_buffer = (player.jump_buffer - dt).max(0.0);
}

pub fn buffer_jump(player: &mut PlayerData) {
    player.jump_buffer = JUMP_BUFFER_TIME;
}

/// Executa o pulo se houver buffer e coyote time disponíveis.
pub fn try_jump(player: &mut PlayerData, body: &mut Body, jump_speed: f32) -> bool {
    if player.jump_buffer > 0.0 && player.coyote > 0.0 {
        jump(body, jump_speed);
        player.jump_buffer = 0.0;
        player.coyote = 0.0;
        player.jump_cut = true;
        true
    } else {
        false
    }
}

/// Integra a velocidade e resolve colisões com o mapa.
pub fn integrate(body: &mut Body, level: &Level, dt: f32) {
    let dx = body.vel.x * dt;
    move_x(body, level, dx);
    let dy = body.vel.y * dt;
    move_y(body, level, dy);
}

fn move_x(body: &mut Body, level: &Level, dx: f32) {
    if dx == 0.0 {
        return;
    }

    body.pos.x += dx;
    if !level.rect_overlaps_solid(body.rect()) {
        return;
    }

    if dx > 0.0 {
        let right = body.right();
        let tx = ((right - SKIN) / TILE_SIZE).floor();
        body.pos.x = tx * TILE_SIZE - body.size.x;
    } else {
        let left = body.pos.x;
        let tx = (left / TILE_SIZE).floor();
        body.pos.x = (tx + 1.0) * TILE_SIZE;
    }
    body.vel.x = 0.0;
}

fn move_y(body: &mut Body, level: &Level, dy: f32) {
    if dy == 0.0 {
        return;
    }

    let previous_bottom = body.bottom();
    body.pos.y += dy;

    if dy >= 0.0 {
        if let Some(top) = landing_surface(body, level, previous_bottom) {
            body.pos.y = top - body.size.y;
            body.vel.y = 0.0;
            body.on_ground = true;
        } else {
            body.on_ground = false;
        }
    } else if let Some(bottom) = ceiling(body, level) {
        body.pos.y = bottom + SKIN;
        body.vel.y = 0.0;
        body.on_ground = false;
    }
}

fn landing_surface(body: &Body, level: &Level, previous_bottom: f32) -> Option<f32> {
    let rect = body.rect();
    let mut landing: Option<f32> = None;

    for ty in rect_tiles(rect, Axis::Y) {
        let tile_top = ty as f32 * TILE_SIZE;
        for tx in rect_tiles(rect, Axis::X) {
            let tile = level.tile_at(tx, ty);
            let is_one_way = tile.is_platform() && previous_bottom <= tile_top + SKIN;
            if !(tile.is_solid() || is_one_way) {
                continue;
            }
            let tile_rect = Rect::new(tx as f32 * TILE_SIZE, tile_top, TILE_SIZE, TILE_SIZE);
            if rect.overlaps(tile_rect) {
                landing = Some(landing.map_or(tile_top, |best: f32| best.min(tile_top)));
            }
        }
    }

    landing
}

fn ceiling(body: &Body, level: &Level) -> Option<f32> {
    let rect = body.rect();
    let mut ceiling: Option<f32> = None;

    for ty in rect_tiles(rect, Axis::Y) {
        for tx in rect_tiles(rect, Axis::X) {
            if !level.tile_at(tx, ty).is_solid() {
                continue;
            }
            let tile_rect = Rect::new(
                tx as f32 * TILE_SIZE,
                ty as f32 * TILE_SIZE,
                TILE_SIZE,
                TILE_SIZE,
            );
            if rect.overlaps(tile_rect) {
                let bottom = tile_rect.bottom();
                ceiling = Some(ceiling.map_or(bottom, |best: f32| best.max(bottom)));
            }
        }
    }

    ceiling
}

enum Axis {
    X,
    Y,
}

fn rect_tiles(rect: Rect, axis: Axis) -> std::ops::RangeInclusive<i32> {
    match axis {
        Axis::X => {
            let first = (rect.left() / TILE_SIZE).floor() as i32;
            let last = ((rect.right() - SKIN) / TILE_SIZE).floor() as i32;
            first..=last
        }
        Axis::Y => {
            let first = (rect.top() / TILE_SIZE).floor() as i32;
            let last = ((rect.bottom() - SKIN) / TILE_SIZE).floor() as i32;
            first..=last
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::Vec2;

    fn level(source: &str) -> Level {
        Level::parse(source).unwrap()
    }

    fn body_at(x: f32, y: f32) -> Body {
        Body::new(Vec2::new(x, y), Vec2::new(24.0, 30.0))
    }

    #[test]
    fn falls_and_lands_on_ground() {
        let level = level("P.\n..\n##\n");
        let mut body = body_at(10.0, 0.0);
        for _ in 0..120 {
            apply_gravity(&mut body, crate::DT);
            integrate(&mut body, &level, crate::DT);
        }
        assert!(body.on_ground);
        assert_eq!(body.bottom(), 2.0 * TILE_SIZE);
        assert_eq!(body.vel.y, 0.0);
    }

    #[test]
    fn does_not_tunnel_through_solid_tile() {
        let level = level("P.\n##\n");
        let mut body = body_at(0.0, 0.0);
        body.vel.y = MAX_FALL_SPEED;
        for _ in 0..10 {
            integrate(&mut body, &level, crate::DT);
        }
        assert_eq!(body.bottom(), TILE_SIZE);
        assert!(body.on_ground);
    }

    #[test]
    fn stops_against_wall() {
        let level = level("P.#\n###\n");
        let mut body = body_at(0.0, 0.0);
        body.vel.x = 300.0;
        for _ in 0..30 {
            integrate(&mut body, &level, crate::DT);
        }
        assert_eq!(body.right(), 2.0 * TILE_SIZE);
        assert_eq!(body.vel.x, 0.0);
    }

    #[test]
    fn one_way_platform_stops_falling_from_above() {
        let level = level("P.\n==\n..\n..\n");
        let mut body = body_at(10.0, 0.0);
        for _ in 0..60 {
            apply_gravity(&mut body, crate::DT);
            integrate(&mut body, &level, crate::DT);
        }
        assert!(body.on_ground);
        assert_eq!(body.bottom(), TILE_SIZE);
    }

    #[test]
    fn one_way_platform_is_ignored_from_below() {
        let level = level("P.\n..\n==\n##\n");
        let mut body = body_at(10.0, 66.0);
        body.vel.y = -600.0;
        integrate(&mut body, &level, crate::DT);
        // A plataforma não bloqueia a subida; o corpo continua acima dela.
        assert!(body.pos.y < 64.0);
        assert!(!body.on_ground);
    }

    #[test]
    fn ceiling_stops_upward_motion() {
        let level = level("##\n..\nP.\n##\n");
        let mut body = body_at(0.0, TILE_SIZE);
        body.vel.y = -600.0;
        integrate(&mut body, &level, crate::DT);
        assert!((body.pos.y - TILE_SIZE).abs() < 0.01);
        assert_eq!(body.vel.y, 0.0);
    }

    #[test]
    fn horizontal_step_accelerates_and_stops() {
        let mut body = body_at(0.0, 0.0);
        for _ in 0..60 {
            horizontal_step(&mut body, 220.0, true, crate::DT);
        }
        assert_eq!(body.vel.x, 220.0);
        for _ in 0..60 {
            horizontal_step(&mut body, 0.0, true, crate::DT);
        }
        assert_eq!(body.vel.x, 0.0);
    }

    #[test]
    fn air_keeps_inertia_without_input() {
        let mut body = body_at(0.0, 0.0);
        body.vel.x = 200.0;
        horizontal_step(&mut body, 0.0, false, crate::DT);
        assert_eq!(body.vel.x, 200.0);
    }

    fn player() -> PlayerData {
        PlayerData::new(
            0,
            Vec2::ZERO,
            crate::entity::Stats::player(Default::default()),
        )
    }

    #[test]
    fn double_tap_starts_running() {
        let mut player = player();
        update_run_state(&mut player, 1, crate::DT);
        assert!(!player.running);
        update_run_state(&mut player, 0, crate::DT);
        update_run_state(&mut player, 1, crate::DT);
        assert!(player.running);
    }

    #[test]
    fn double_tap_outside_window_does_not_run() {
        let mut player = player();
        update_run_state(&mut player, 1, crate::DT);
        update_run_state(&mut player, 0, crate::DT);
        for _ in 0..30 {
            update_run_state(&mut player, 0, crate::DT);
        }
        update_run_state(&mut player, 1, crate::DT);
        assert!(!player.running);
    }

    #[test]
    fn holding_direction_keeps_running_and_release_cancels() {
        let mut player = player();
        update_run_state(&mut player, 1, crate::DT);
        update_run_state(&mut player, 0, crate::DT);
        update_run_state(&mut player, 1, crate::DT);
        assert!(player.running);
        for _ in 0..10 {
            update_run_state(&mut player, 1, crate::DT);
        }
        assert!(player.running);
        update_run_state(&mut player, 0, crate::DT);
        assert!(!player.running);
    }

    #[test]
    fn reversing_cancels_run() {
        let mut player = player();
        update_run_state(&mut player, 1, crate::DT);
        update_run_state(&mut player, 0, crate::DT);
        update_run_state(&mut player, 1, crate::DT);
        assert!(player.running);
        update_run_state(&mut player, -1, crate::DT);
        assert!(!player.running);
    }

    #[test]
    fn jump_buffer_waits_for_landing() {
        let mut player = player();
        let mut body = body_at(0.0, 0.0);
        buffer_jump(&mut player);
        player.coyote = 0.0;
        assert!(!try_jump(&mut player, &mut body, 560.0));
        // Simula o pouso: coyote é renovado e o buffer ainda está ativo.
        update_timers(&mut player, true, crate::DT);
        assert!(try_jump(&mut player, &mut body, 560.0));
        assert_eq!(body.vel.y, -560.0);
    }

    #[test]
    fn coyote_allows_jump_after_leaving_ground() {
        let mut player = player();
        let mut body = body_at(0.0, 0.0);
        update_timers(&mut player, true, crate::DT);
        update_timers(&mut player, false, crate::DT);
        buffer_jump(&mut player);
        assert!(try_jump(&mut player, &mut body, 560.0));
    }

    #[test]
    fn jump_cut_halves_ascent_once() {
        let mut player = player();
        let mut body = body_at(0.0, 0.0);
        jump(&mut body, 560.0);
        player.jump_cut = true;
        cut_jump(&mut body, &mut player);
        assert_eq!(body.vel.y, -280.0);
        cut_jump(&mut body, &mut player);
        assert_eq!(body.vel.y, -280.0);
    }
}
