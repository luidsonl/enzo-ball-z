//! Playable **Valentina** actor.

use crate::math::Vec2;
use crate::mechanics::Locomotion;
use crate::skills::Skills;

use super::player::{PlayableCharacter, PlayerData, PlayerId, VisualProfile};
use crate::entities::base::attributes::{Attributes, Stats};
use crate::entities::base::components::{Health, TeamId};
use crate::entities::base::controller::Controller;
use crate::entities::base::entity::EntityKind;
use crate::entities::base::equipment::{EquipSlot, Equipment};
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
        let attributes = Attributes::default();
        let stats = Stats::player(attributes);
        Self {
            kind: EntityKind::Player(PlayerData::new(
                player_id,
                PlayableCharacter::Valentina,
                VisualProfile::default(),
                pos,
                stats,
            )),
            controller: Controller::player(),
            team: TeamId::PLAYERS,
            pos,
            size: VALENTINA_SIZE,
            health: Health::full(stats.max_health),
            stats,
            attributes,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            locomotion: Locomotion::player(),
            skills: Skills::player_default(),
            velocity: Vec2::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntityStore;
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
