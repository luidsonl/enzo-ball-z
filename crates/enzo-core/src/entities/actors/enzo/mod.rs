//! Playable **Enzo** actor.

use crate::math::Vec2;

use super::humanoid::PlayableHumanoidConfig;
use super::player::{PlayableCharacter, PlayerId};
use crate::entities::spec::EntitySpec;

/// Enzo body size.
pub const ENZO_SIZE: Vec2 = Vec2::new(24.0, 30.0);

/// Visual/animation contract (pure data). Rendering resolves frames from this.
pub mod visual {
    pub const SHEET: &str = "assets/entities/enzo/sheet.png";
    pub const ATLAS: &str = "assets/entities/enzo/atlas.json";
    pub const CELL: f32 = 32.0;
    pub const RIG: &str = "assets/entities/enzo/rig.json";
    pub const CLIPS: &[&str] = &[
        "idle",
        "walk",
        "run",
        "jump",
        "fall",
        "attack_1",
        "attack_2",
        "attack_3",
        "cast",
        "hurt",
        "sixseven_6",
        "sixseven_7",
    ];
}

impl EntitySpec {
    /// Creates an Enzo player. `player_id` is stable and indexes per-tick commands.
    pub fn player(player_id: PlayerId, pos: Vec2) -> Self {
        Self::enzo(player_id, pos)
    }

    /// Creates an Enzo playable character.
    pub fn enzo(player_id: PlayerId, pos: Vec2) -> Self {
        Self::playable_humanoid(
            player_id,
            pos,
            PlayableHumanoidConfig::new(PlayableCharacter::Enzo, ENZO_SIZE),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{Attributes, EntityStore, Stats};
    use crate::input::ActionSlot;
    use crate::skills::SkillId;

    #[test]
    fn enzo_uses_enzo_components() {
        let mut store = EntityStore::new();
        let id = store.spawn(EntitySpec::player(0, Vec2::ZERO));
        let entity = store.get(id).unwrap();

        assert_eq!(entity.body.size, ENZO_SIZE);
        assert_eq!(entity.player().unwrap().player_id, 0);
        assert_eq!(entity.stats, Stats::player(Attributes::default()));
        assert!(entity.locomotion.has_walk());
        assert!(entity.locomotion.has_sixseven());
        assert!(entity.skills.has(SkillId::Jump));
        assert_eq!(
            entity.skills.loadout.get(ActionSlot::Slot1),
            Some(SkillId::Jump)
        );
        assert_eq!(entity.controller.player_input().unwrap().move_x, 0);
    }
}
