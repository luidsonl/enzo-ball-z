//! Entidade jogável **Enzo**.

use crate::math::Vec2;
use crate::mechanics::Locomotion;
use crate::skills::Skills;

use super::attributes::{Attributes, Stats};
use super::components::{Health, TeamId};
use super::controller::Controller;
use super::entity::EntityKind;
use super::equipment::{EquipSlot, Equipment};
use super::player::{PlayerData, PlayerId};
use super::spec::EntitySpec;

/// Corpo do Enzo.
pub const ENZO_SIZE: Vec2 = Vec2::new(24.0, 30.0);

impl EntitySpec {
    /// Cria um jogador Enzo. `player_id` é estável e indexa os comandos do tick.
    pub fn player(player_id: PlayerId, pos: Vec2) -> Self {
        let attributes = Attributes::default();
        let stats = Stats::player(attributes);
        Self {
            kind: EntityKind::Player(PlayerData::new(player_id, pos, stats)),
            controller: Controller::player(),
            team: TeamId::PLAYERS,
            pos,
            size: ENZO_SIZE,
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
