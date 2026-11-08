//! Logical animation/visual state contract.
//!
//! Core resolves gameplay state into stable, deterministic visual intents.
//! No image paths, frame indices, or rendering details live here.

use crate::input::SixSevenKey;

/// High-level locomotion/pose for rendering.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum LocomotionPose {
    #[default]
    Idle,
    Walk,
    Run,
    Crouch,
    JumpRise,
    JumpFall,
    Dash,
    Fly,
    Swim,
    Climb,
}

impl LocomotionPose {
    pub const fn clip_name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Walk => "walk",
            Self::Run => "run",
            Self::Crouch => "crouch",
            Self::JumpRise => "jump_rise",
            Self::JumpFall => "jump_fall",
            Self::Dash => "dash",
            Self::Fly => "fly",
            Self::Swim => "swim",
            Self::Climb => "climb",
        }
    }
}

/// Action overlay that can take priority over locomotion.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum ActionPose {
    #[default]
    None,
    Melee(u8),
    Kick,
    Cast,
    Shout,
    Hurt,
    Dying,
    Dead,
    SixSeven(SixSevenHand),
}

impl ActionPose {
    pub fn clip_name(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Melee(step) => match step {
                1 => "attack_1",
                2 => "attack_2",
                _ => "attack_3",
            },
            Self::Kick => "kick",
            Self::Cast => "cast",
            Self::Shout => "shout",
            Self::Hurt => "hurt",
            Self::Dying => "dying",
            Self::Dead => "dead",
            Self::SixSeven(SixSevenHand::A) => "sixseven_a",
            Self::SixSeven(SixSevenHand::B) => "sixseven_b",
        }
    }
}

/// Which hand/pose is shown during SixSeven aura farming.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SixSevenHand {
    #[default]
    A,
    B,
}

/// Resolved logical animation state for a character entity.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(default))]
pub struct AnimationState {
    pub pose: LocomotionPose,
    pub action: ActionPose,
    pub facing: i8,
    pub on_ground: bool,
    pub farming: bool,
}

impl AnimationState {
    /// Deterministically resolves animation from gameplay-visible state.
    /// `time` is used only for stable alternating poses (e.g. SixSeven) and must
    /// come from a deterministic source when used in simulation-facing contexts;
    /// renderers may pass visual time for presentation.
    #[allow(clippy::too_many_arguments)]
    pub fn resolve(
        alive: bool,
        facing: i8,
        on_ground: bool,
        vel_x: f32,
        vel_y: f32,
        running: bool,
        dashing: bool,
        flying: bool,
        swimming: bool,
        hurt: bool,
        melee_step: u8,
        melee_active: bool,
        kick_active: bool,
        cast_active: bool,
        farming: bool,
        sixseven_last: Option<SixSevenKey>,
        anim_time: f32,
    ) -> Self {
        Self::resolve_with_pose(
            alive,
            facing,
            on_ground,
            vel_x,
            vel_y,
            running,
            dashing,
            flying,
            swimming,
            false,
            false,
            hurt,
            melee_step,
            melee_active,
            kick_active,
            cast_active,
            false,
            farming,
            sixseven_last,
            anim_time,
        )
    }

    /// Extended resolver with additional locomotion intents (e.g. crouch).
    #[allow(clippy::too_many_arguments)]
    pub fn resolve_with_pose(
        alive: bool,
        facing: i8,
        on_ground: bool,
        vel_x: f32,
        vel_y: f32,
        running: bool,
        dashing: bool,
        flying: bool,
        swimming: bool,
        climbing: bool,
        crouching: bool,
        hurt: bool,
        melee_step: u8,
        melee_active: bool,
        kick_active: bool,
        cast_active: bool,
        shout_active: bool,
        farming: bool,
        sixseven_last: Option<SixSevenKey>,
        anim_time: f32,
    ) -> Self {
        let pose = if !alive {
            LocomotionPose::Idle
        } else if climbing {
            LocomotionPose::Climb
        } else if swimming {
            LocomotionPose::Swim
        } else if dashing {
            LocomotionPose::Dash
        } else if flying {
            LocomotionPose::Fly
        } else if !on_ground {
            if vel_y < -20.0 {
                LocomotionPose::JumpRise
            } else {
                LocomotionPose::JumpFall
            }
        } else if crouching {
            LocomotionPose::Crouch
        } else if vel_x.abs() > 20.0 {
            if running {
                LocomotionPose::Run
            } else {
                LocomotionPose::Walk
            }
        } else {
            LocomotionPose::Idle
        };

        let action = if !alive {
            ActionPose::Dead
        } else if hurt {
            ActionPose::Hurt
        } else if shout_active {
            ActionPose::Shout
        } else if cast_active {
            ActionPose::Cast
        } else if kick_active {
            ActionPose::Kick
        } else if melee_active {
            ActionPose::Melee(melee_step.max(1))
        } else if farming {
            let hand = if ((anim_time * 10.0).floor() as i32 & 1) == 0 {
                SixSevenHand::A
            } else {
                SixSevenHand::B
            };
            let _ = sixseven_last; // available for future per-key poses
            ActionPose::SixSeven(hand)
        } else {
            ActionPose::None
        };

        Self {
            pose,
            action,
            facing: if facing >= 0 { 1 } else { -1 },
            on_ground,
            farming,
        }
    }

    /// Clip key for rig/asset resolution. Action clips take priority over locomotion.
    pub fn clip_name(self) -> &'static str {
        if self.action == ActionPose::None {
            self.pose.clip_name()
        } else {
            self.action.clip_name()
        }
    }

    pub fn is_dead(self) -> bool {
        matches!(self.action, ActionPose::Dead | ActionPose::Dying)
    }

    pub fn is_sixseven(self) -> bool {
        matches!(self.action, ActionPose::SixSeven(_))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn resolve_default() -> AnimationState {
        AnimationState::resolve(
            true, 1, true, 0.0, 0.0, false, false, false, false, false, 0, false, false, false,
            false, None, 0.0,
        )
    }

    #[test]
    fn idle_by_default() {
        let state = resolve_default();
        assert_eq!(state.pose, LocomotionPose::Idle);
        assert_eq!(state.action, ActionPose::None);
    }

    #[test]
    fn airborne_uses_jump_pose() {
        let rise = AnimationState::resolve(
            true, 1, false, 0.0, -100.0, false, false, false, false, false, 0, false, false, false,
            false, None, 0.0,
        );
        assert_eq!(rise.pose, LocomotionPose::JumpRise);
        let fall = AnimationState::resolve(
            true, 1, false, 0.0, 100.0, false, false, false, false, false, 0, false, false, false,
            false, None, 0.0,
        );
        assert_eq!(fall.pose, LocomotionPose::JumpFall);
    }

    #[test]
    fn action_priority_prefers_damage_and_attacks() {
        let hurt = AnimationState::resolve(
            true, 1, true, 0.0, 0.0, false, false, false, false, true, 0, true, false, false, true,
            None, 0.0,
        );
        assert_eq!(hurt.action, ActionPose::Hurt);
        let melee = AnimationState::resolve(
            true, 1, true, 0.0, 0.0, false, false, false, false, false, 2, true, false, false,
            true, None, 0.0,
        );
        assert_eq!(melee.action, ActionPose::Melee(2));
        let dead = AnimationState::resolve(
            false, 1, true, 0.0, 0.0, false, false, false, false, false, 0, false, false, false,
            false, None, 0.0,
        );
        assert_eq!(dead.action, ActionPose::Dead);
    }

    #[test]
    fn sixseven_alternates_hands() {
        let a = AnimationState::resolve(
            true,
            1,
            true,
            0.0,
            0.0,
            false,
            false,
            false,
            false,
            false,
            0,
            false,
            false,
            false,
            true,
            Some(SixSevenKey::Six),
            0.0,
        );
        let b = AnimationState::resolve(
            true,
            1,
            true,
            0.0,
            0.0,
            false,
            false,
            false,
            false,
            false,
            0,
            false,
            false,
            false,
            true,
            Some(SixSevenKey::Seven),
            0.11,
        );
        assert_eq!(a.action, ActionPose::SixSeven(SixSevenHand::A));
        assert_eq!(b.action, ActionPose::SixSeven(SixSevenHand::B));
    }

    #[test]
    fn clip_names_follow_action_priority_and_poses() {
        let mut state = resolve_default();
        state.pose = LocomotionPose::Crouch;
        assert_eq!(state.clip_name(), "crouch");
        let melee = AnimationState::resolve(
            true, 1, true, 0.0, 0.0, false, false, false, false, false, 2, true, false, false,
            false, None, 0.0,
        );
        assert_eq!(melee.clip_name(), "attack_2");
        let swim = AnimationState::resolve_with_pose(
            true, 1, false, 0.0, 0.0, false, false, false, true, false, false, false, 0, false,
            false, false, false, false, None, 0.0,
        );
        assert_eq!(swim.clip_name(), "swim");
        let climb = AnimationState::resolve_with_pose(
            true, 1, false, 0.0, -10.0, false, false, false, false, true, false, false, 0, false,
            false, false, false, false, None, 0.0,
        );
        assert_eq!(climb.clip_name(), "climb");
        let shout = AnimationState::resolve_with_pose(
            true, 1, true, 0.0, 0.0, false, false, false, false, false, false, false, 0, false,
            false, false, true, false, None, 0.0,
        );
        assert_eq!(shout.clip_name(), "shout");
    }
}
