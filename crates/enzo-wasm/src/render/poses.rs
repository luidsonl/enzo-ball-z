//! Escolha de frames de animação a partir do estado visual.

use enzo_core::entities::ItemKind;

use crate::sprites::{enzo, item, slime};

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
