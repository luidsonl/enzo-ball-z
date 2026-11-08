//! Animation and drawing for the playable Valentina character.

use enzo_core::entities::{ActionPose, AnimationState, LocomotionPose, SixSevenHand};
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::sprites::{self, Sprites};

use super::super::skills;
use super::compositor;

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
    pub dead: bool,
}

impl Pose {
    pub fn from_snapshot(entity: &EntitySnapshot, player: &PlayerSnapshot) -> Self {
        let casting = player.slots.iter().any(|slot| {
            slot.active
                && matches!(
                    slot.skill,
                    Some(
                        enzo_core::skills::SkillId::Mogging
                            | enzo_core::skills::SkillId::NutellaShot
                            | enzo_core::skills::SkillId::Shout
                    )
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
            dead: entity.health <= 0.0,
        }
    }

    fn to_animation(self, time: f32) -> AnimationState {
        AnimationState::resolve(
            !self.dead,
            if self.vel_x < 0.0 { -1 } else { 1 },
            self.on_ground,
            self.vel_x,
            self.vel_y,
            false,
            self.dashing,
            false,
            false,
            self.iframes > 0.0 && !self.dead,
            1,
            self.melee_timer > 0.0,
            self.melee_timer > 0.0, // Valentina uses kick for melee in legacy behavior
            self.casting,
            self.farming,
            None,
            time,
        )
    }
}

pub fn frame(pose: Pose, time: f32) -> usize {
    frame_from_animation(pose.to_animation(time), time)
}

pub fn frame_from_animation(anim: AnimationState, time: f32) -> usize {
    use sprites::valentina as f;
    match anim.action {
        ActionPose::Dead | ActionPose::Dying | ActionPose::Hurt => f::HURT,
        ActionPose::Cast => f::CAST,
        ActionPose::Kick | ActionPose::Melee(_) => f::KICK,
        ActionPose::SixSeven(hand) => match hand {
            SixSevenHand::A => f::SIXSEVEN_A,
            SixSevenHand::B => f::SIXSEVEN_B,
        },
        ActionPose::None => match anim.pose {
            LocomotionPose::Dash
            | LocomotionPose::Fly
            | LocomotionPose::JumpRise
            | LocomotionPose::JumpFall => f::JUMP,
            LocomotionPose::Swim => {
                if ((time * 6.0) as usize).is_multiple_of(2) {
                    f::WALK_1
                } else {
                    f::WALK_2
                }
            }
            LocomotionPose::Crouch => f::IDLE,
            LocomotionPose::Walk | LocomotionPose::Run => {
                if ((time * 8.0) as usize).is_multiple_of(2) {
                    f::WALK_1
                } else {
                    f::WALK_2
                }
            }
            LocomotionPose::Idle => f::IDLE,
        },
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
    let mut tint = if blink {
        Color::from_rgba(255, 255, 255, 90)
    } else if anim.is_dead() {
        Color::from_rgba(220, 220, 240, 180)
    } else {
        WHITE
    };
    tint = compositor::profile_tint(player, tint);

    let dest = Rect::new(center.x - 16.0, pos.y + size.y - 32.0, 32.0, 32.0);
    let flip = entity.facing < 0;
    sprites.draw_valentina(frame_from_animation(anim, time), dest, flip, tint);
    compositor::draw_equipped_layers(sprites, player, dest, flip, tint);

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
            dead: false,
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
