//! Câmera de plataforma: segue o jogador, olha na direção do movimento e trava
//! nas bordas do mapa.
//!
//! A câmera é puramente visual. Ela não altera o snapshot nem a simulação.

use enzo_core::snapshot::{SnapshotKind, WorldSnapshot};
use macroquad::prelude::{vec2, Camera2D, Rect, Vec2};

use crate::convert::to_vec2;

/// Altura lógica da tela em unidades de mundo. Em 720p isso dá escala 2x.
const VIRTUAL_HEIGHT: f32 = 360.0;
const FOLLOW_SPEED: f32 = 8.0;
const LOOKAHEAD: f32 = 38.0;
const LOOKAHEAD_VELOCITY: f32 = 20.0;
const MAX_SHAKE: f32 = 14.0;

pub struct GameCamera {
    pub center: Vec2,
    pub view: Vec2,
    shake: f32,
    shake_clock: f32,
    initialized: bool,
}

impl GameCamera {
    pub fn new() -> Self {
        Self {
            center: Vec2::ZERO,
            view: vec2(640.0, VIRTUAL_HEIGHT),
            shake: 0.0,
            shake_clock: 0.0,
            initialized: false,
        }
    }

    /// Recalcula o campo de visão a partir do tamanho real da tela.
    pub fn resize(&mut self, screen: Vec2) {
        let scale = (screen.y / VIRTUAL_HEIGHT).max(1.0);
        self.view = screen / scale;
    }

    pub fn update(&mut self, dt: f32, snapshot: &WorldSnapshot) {
        let target = self.target(snapshot);
        let target = clamp_center(
            target,
            snapshot.level_width,
            snapshot.level_height,
            self.view,
        );

        if self.initialized {
            let smoothing = 1.0 - (-FOLLOW_SPEED * dt).exp();
            self.center += (target - self.center) * smoothing;
        } else {
            self.center = target;
            self.initialized = true;
        }

        self.shake = (self.shake - dt * 22.0).max(0.0);
        self.shake_clock += dt;
    }

    pub fn add_shake(&mut self, amount: f32) {
        self.shake = (self.shake + amount).min(MAX_SHAKE);
    }

    pub fn reset(&mut self) {
        self.initialized = false;
        self.shake = 0.0;
        self.shake_clock = 0.0;
    }

    pub fn view_rect(&self) -> Rect {
        Rect::new(
            self.center.x - self.view.x * 0.5,
            self.center.y - self.view.y * 0.5,
            self.view.x,
            self.view.y,
        )
    }

    pub fn camera(&self) -> Camera2D {
        let offset = if self.shake > 0.0 {
            vec2(
                (self.shake_clock * 91.0).sin(),
                (self.shake_clock * 73.0).cos(),
            ) * self.shake
        } else {
            Vec2::ZERO
        };
        let view = self.view_rect();
        world_camera(Rect::new(
            view.x + offset.x,
            view.y + offset.y,
            view.w,
            view.h,
        ))
    }

    fn target(&self, snapshot: &WorldSnapshot) -> Vec2 {
        let Some((pos, size, facing, velocity)) = player_focus(snapshot) else {
            return self.center;
        };
        let center = pos + size * 0.5;
        let look =
            facing as f32 * LOOKAHEAD + (velocity.x / 340.0).clamp(-1.0, 1.0) * LOOKAHEAD_VELOCITY;
        vec2(center.x + look, center.y - 24.0)
    }
}

impl Default for GameCamera {
    fn default() -> Self {
        Self::new()
    }
}

/// Câmera com Y para baixo, igual ao core.
///
/// `Camera2D::from_display_rect` inverte o eixo Y (`zoom.y` negativo), o que
/// desenharia o mundo de cabeça para baixo. Construímos a câmera manualmente
/// para preservar a convenção do core: Y cresce para baixo.
fn world_camera(rect: Rect) -> Camera2D {
    Camera2D {
        target: vec2(rect.x + rect.w * 0.5, rect.y + rect.h * 0.5),
        zoom: vec2(2.0 / rect.w, 2.0 / rect.h),
        ..Default::default()
    }
}

fn player_focus(snapshot: &WorldSnapshot) -> Option<(Vec2, Vec2, i8, Vec2)> {
    snapshot.entities.iter().find_map(|entity| {
        if matches!(entity.kind, SnapshotKind::Player(_)) {
            Some((
                to_vec2(entity.pos),
                to_vec2(entity.size),
                entity.facing,
                to_vec2(entity.vel),
            ))
        } else {
            None
        }
    })
}

/// Mantém a câmera dentro do mapa. Se o mapa for menor que a tela, centraliza.
pub fn clamp_center(center: Vec2, level_width: f32, level_height: f32, view: Vec2) -> Vec2 {
    let half = view * 0.5;
    let x = if level_width <= view.x {
        level_width * 0.5
    } else {
        center.x.clamp(half.x, level_width - half.x)
    };
    let y = if level_height <= view.y {
        level_height * 0.5
    } else {
        center.y.clamp(half.y, level_height - half.y)
    };
    vec2(x, y)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamp_keeps_camera_inside_level() {
        let view = vec2(640.0, 360.0);
        let clamped = clamp_center(vec2(-100.0, -100.0), 3200.0, 704.0, view);
        assert_eq!(clamped, vec2(320.0, 180.0));

        let clamped = clamp_center(vec2(5000.0, 5000.0), 3200.0, 704.0, view);
        assert_eq!(clamped, vec2(2880.0, 524.0));
    }

    #[test]
    fn clamp_centers_small_levels() {
        let view = vec2(640.0, 360.0);
        let clamped = clamp_center(vec2(10.0, 10.0), 320.0, 180.0, view);
        assert_eq!(clamped, vec2(160.0, 90.0));
    }

    #[test]
    fn view_resize_keeps_virtual_height() {
        let mut camera = GameCamera::new();
        camera.resize(vec2(1280.0, 720.0));
        assert!((camera.view.y - VIRTUAL_HEIGHT).abs() < 0.001);
        assert!((camera.view.x - 640.0).abs() < 0.001);

        camera.resize(vec2(1920.0, 1080.0));
        assert!((camera.view.y - VIRTUAL_HEIGHT).abs() < 0.001);
        assert!((camera.view.x - 640.0).abs() < 0.001);
    }

    #[test]
    fn world_camera_keeps_y_pointing_down() {
        let camera = world_camera(Rect::new(10.0, 20.0, 640.0, 360.0));
        assert_eq!(camera.target, vec2(330.0, 200.0));
        assert!(camera.zoom.y > 0.0);
        assert_eq!(camera.zoom, vec2(2.0 / 640.0, 2.0 / 360.0));
    }
}
