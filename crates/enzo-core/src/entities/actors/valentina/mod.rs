//! Playable **Valentina** actor.

use crate::math::Vec2;

use super::humanoid::PlayableHumanoidConfig;
use super::player::{PlayableCharacter, PlayerId};
use crate::entities::spec::EntitySpec;

/// Valentina body size.
pub const VALENTINA_SIZE: Vec2 = Vec2::new(23.0, 30.0);

/// Visual/animation contract (pure data). Rendering resolves frames from this.
pub mod visual {
    pub const SHEET: &str = "assets/entities/valentina/sheet.png";
    pub const ATLAS: &str = "assets/entities/valentina/atlas.json";
    pub const CELL: f32 = 32.0;
    pub const RIG: &str = "assets/entities/valentina/rig.json";
    /// Valentina can have different animation clips/timing than Enzo.
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
        "kick",
        "hurt",
        "sixseven_6",
        "sixseven_7",
    ];
}

impl EntitySpec {
    /// Creates a Valentina playable character.
    pub fn valentina(player_id: PlayerId, pos: Vec2) -> Self {
        Self::playable_humanoid(
            player_id,
            pos,
            PlayableHumanoidConfig::new(PlayableCharacter::Valentina, VALENTINA_SIZE),
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
    fn valentina_is_a_playable_entity() {
        let mut store = EntityStore::new();
        let id = store.spawn(EntitySpec::valentina(0, Vec2::ZERO));
        let entity = store.get(id).unwrap();

        assert_eq!(entity.body.size, VALENTINA_SIZE);
        assert_eq!(entity.player().unwrap().player_id, 0);
        assert_eq!(
            entity.player().unwrap().character,
            PlayableCharacter::Valentina
        );
        assert_eq!(entity.stats, Stats::player(Attributes::default()));
        assert!(entity.locomotion.has_walk());
        assert!(entity.skills.has(SkillId::Jump));
        assert_eq!(
            entity.skills.loadout.get(ActionSlot::Slot1),
            Some(SkillId::Jump)
        );
    }
}
