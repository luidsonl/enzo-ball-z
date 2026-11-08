//! Animation and drawing for the playable Valentina character.

use enzo_core::skills::SkillId;
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
    pub iframes: f32,
    pub casting: bool,
}

impl Pose {
    pub fn from_snapshot(entity: &EntitySnapshot, player: &PlayerSnapshot) -> Self {
        let casting = player.slots.iter().any(|slot| {
            slot.active
                && matches!(
                    slot.skill,
                    Some(SkillId::Mogging | SkillId::NutellaShot | SkillId::Shout)
                )
        });
        Self {
            on_ground: entity.on_ground,
            vel_x: entity.vel.x,
            vel_y: entity.vel.y,
            dashing: entity.dashing,
            farming: player.farming,
            melee_timer: player.melee_timer,
            iframes: entity.iframes,
            casting,
        }
    }
}

pub fn frame(pose: Pose, time: f32) -> usize {
    use sprites::valentina as f;
    if pose.iframes > 0.0 {
        return f::HURT;
    }
    if pose.casting {
        return f::CAST;
    }
    if pose.melee_timer > 0.0 {
        return f::KICK;
    }
    if pose.farming {
        // Alternating hand poses for SixSeven aura farming.
        return if ((time * 10.0) as usize).is_multiple_of(2) {
            f::SIXSEVEN_A
        } else {
            f::SIXSEVEN_B
        };
    }
    if pose.dashing {
        return f::JUMP;
    }
    if !pose.on_ground {
        return f::JUMP;
    }
    if pose.vel_x.abs() > 20.0 {
        return if ((time * 8.0) as usize).is_multiple_of(2) {
            f::WALK_1
        } else {
            f::WALK_2
        };
    }
    f::IDLE
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

    // Shadow
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
            Color::from_rgba(255, 150, 220, 22),
        );
    }

    let blink = entity.iframes > 0.0 && (time * 24.0) as i32 % 2 == 0;
    let tint = if blink {
        Color::from_rgba(255, 255, 255, 90)
    } else {
        WHITE
    };
    sprites.draw_valentina(
        frame(pose, time),
        Rect::new(center.x - 16.0, pos.y + size.y - 32.0, 32.0, 32.0),
        entity.facing < 0,
        tint,
    );

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
            iframes: 0.0,
            casting: false,
        }
    }

    #[test]
    fn farming_uses_alternating_sixseven_frames() {
        let farming = Pose {
            farming: true,
            ..pose()
        };
        assert_eq!(frame(farming, 0.0), sprites::valentina::SIXSEVEN_A);
        assert_eq!(frame(farming, 0.11), sprites::valentina::SIXSEVEN_B);
    }

    #[test]
    fn melee_uses_kick_and_cast_has_priority() {
        assert_eq!(
            frame(
                Pose {
                    melee_timer: 0.1,
                    ..pose()
                },
                0.0
            ),
            sprites::valentina::KICK
        );
        assert_eq!(
            frame(
                Pose {
                    casting: true,
                    melee_timer: 0.1,
                    ..pose()
                },
                0.0
            ),
            sprites::valentina::CAST
        );
    }
}
