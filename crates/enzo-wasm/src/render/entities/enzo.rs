//! Animação e desenho do personagem Enzo.

use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::sprites::{self, Sprites};

use super::super::skills;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pose {
    pub on_ground: bool,
    pub vel_x: f32,
    pub vel_y: f32,
    pub dashing: bool,
    pub farming: bool,
    pub melee_timer: f32,
    pub melee_step: u8,
    pub iframes: f32,
}

impl Pose {
    pub fn from_snapshot(entity: &EntitySnapshot, player: &PlayerSnapshot) -> Self {
        Self {
            on_ground: entity.on_ground,
            vel_x: entity.vel.x,
            vel_y: entity.vel.y,
            dashing: entity.dashing,
            farming: player.farming,
            melee_timer: player.melee_timer,
            melee_step: player.melee_step,
            iframes: entity.iframes,
        }
    }
}

pub fn frame(pose: Pose, time: f32) -> usize {
    use sprites::enzo as f;
    if pose.melee_timer > 0.0 {
        return if pose.melee_step >= 3 {
            f::ATTACK_B
        } else {
            f::ATTACK_A
        };
    }
    if pose.iframes > 0.0 {
        return f::HURT;
    }
    if pose.dashing {
        return f::DASH;
    }
    if pose.farming {
        return if ((time * 10.0) as usize).is_multiple_of(2) {
            f::FARM_A
        } else {
            f::FARM_B
        };
    }
    if !pose.on_ground {
        return if pose.vel_y < 0.0 { f::JUMP } else { f::FALL };
    }
    if pose.vel_x.abs() > 20.0 {
        let phase = (time * (4.0 + pose.vel_x.abs() / 80.0)) as usize % 4;
        return [f::RUN_A, f::RUN_B, f::RUN_C, f::RUN_D][phase];
    }
    if ((time * 2.5) as usize).is_multiple_of(2) {
        f::IDLE_A
    } else {
        f::IDLE_B
    }
}

pub fn draw(
    sprites: &Sprites,
    entity: &EntitySnapshot,
    player: &PlayerSnapshot,
    pos: Vec2,
    time: f32,
) {
    let size = vec2(entity.size.x, entity.size.y);
    let center = pos + size * 0.5;
    let pose = Pose::from_snapshot(entity, player);

    // sombra
    draw_ellipse(
        pos.x + size.x * 0.5,
        pos.y + size.y - 2.0,
        size.x * 0.475,
        3.5,
        0.0,
        Color::from_rgba(12, 10, 22, 70),
    );

    if player.farming {
        let pulse = 0.5 + 0.5 * (time * 8.0).sin();
        draw_circle(
            center.x,
            center.y,
            20.0 + pulse * 5.0,
            Color::from_rgba(94, 242, 255, 22),
        );
    }

    let blink = entity.iframes > 0.0 && (time * 24.0) as i32 % 2 == 0;
    let tint = if blink {
        Color::from_rgba(255, 255, 255, 90)
    } else {
        WHITE
    };
    sprites.draw_enzo(
        frame(pose, time),
        Rect::new(center.x - 16.0, pos.y + size.y - 32.0, 32.0, 32.0),
        entity.facing < 0,
        tint,
    );

    // Overlay de ataque é responsabilidade da skill.
    skills::draw_player_overlays(sprites, entity, player, center);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pose() -> Pose {
        Pose {
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
        assert_eq!(frame(pose(), 0.0), sprites::enzo::IDLE_A);
        assert_eq!(frame(pose(), 0.5), sprites::enzo::IDLE_B);
        let running = Pose {
            vel_x: 220.0,
            ..pose()
        };
        assert!([
            sprites::enzo::RUN_A,
            sprites::enzo::RUN_B,
            sprites::enzo::RUN_C,
            sprites::enzo::RUN_D
        ]
        .contains(&frame(running, 0.0)));
    }

    #[test]
    fn air_dash_farm_and_attack_have_priority() {
        assert_eq!(
            frame(
                Pose {
                    on_ground: false,
                    vel_y: -100.0,
                    ..pose()
                },
                0.0
            ),
            sprites::enzo::JUMP
        );
        assert_eq!(
            frame(
                Pose {
                    dashing: true,
                    ..pose()
                },
                0.0
            ),
            sprites::enzo::DASH
        );
    }

    #[test]
    fn hurt_frame_wins_over_movement() {
        let hurt = Pose {
            iframes: 0.5,
            vel_x: 220.0,
            ..pose()
        };
        assert_eq!(frame(hurt, 0.0), sprites::enzo::HURT);
    }
}
