//! Entidade **slime**.

use crate::ai::Ai;
use crate::math::Vec2;
use crate::mechanics::Locomotion;
use crate::skills::Skills;

use super::enemy::{EnemyData, EnemySpecies};
use crate::entities::base::attributes::{Attributes, Stats};
use crate::entities::base::components::{Health, TeamId};
use crate::entities::base::controller::Controller;
use crate::entities::base::entity::EntityKind;
use crate::entities::base::equipment::{EquipSlot, Equipment};
use crate::entities::spec::EntitySpec;

/// Corpo da slime.
pub const SLIME_SIZE: Vec2 = Vec2::new(26.0, 26.0);
pub const SLIME_XP_REWARD: f32 = 25.0;
pub const SLIME_CONTACT_DAMAGE: f32 = 10.0;

/// Visual/animation contract (pure data). Rendering resolves frames from this.
pub mod visual {
    pub const SHEET: &str = "assets/entities/slime/sheet.png";
    pub const CELL: f32 = 32.0;
    pub const WALK_A: usize = 0;
    pub const WALK_B: usize = 1;
    pub const STRETCH: usize = 2;
    pub const HURT: usize = 3;
}

impl Stats {
    pub fn slime() -> Self {
        Self {
            walk_speed: 60.0,
            run_speed: 60.0,
            jump_speed: 0.0,
            strength: 0.0,
            max_health: 30.0,
            max_aura: 0.0,
            mogging_damage: 0.0,
        }
    }
}

impl EntitySpec {
    pub fn slime(pos: Vec2) -> Self {
        let stats = Stats::slime();
        Self {
            kind: EntityKind::Enemy(EnemyData {
                species: EnemySpecies::Slime,
                xp_reward: SLIME_XP_REWARD,
                contact_damage: SLIME_CONTACT_DAMAGE,
                stun: 0.0,
                xp_awarded: false,
            }),
            controller: Controller::from_ai(Ai::slime()),
            team: TeamId::ENEMIES,
            pos,
            size: SLIME_SIZE,
            health: Health::full(stats.max_health),
            stats,
            attributes: Attributes::ZERO,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            locomotion: Locomotion::enemy(),
            skills: Skills::enemy(),
            velocity: Vec2::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntityStore;

    #[test]
    fn slime_has_slime_components() {
        let mut store = EntityStore::new();
        let id = store.spawn(EntitySpec::slime(Vec2::ZERO));
        let entity = store.get(id).unwrap();

        assert_eq!(entity.body.size, SLIME_SIZE);
        assert_eq!(entity.enemy().unwrap().xp_reward, SLIME_XP_REWARD);
        assert_eq!(entity.stats, Stats::slime());
        assert!(entity.locomotion.has_walk());
        assert!(!entity.locomotion.has_sixseven());
        assert!(entity.skills.loadout.is_empty());
        assert!(entity.ai().is_some());
    }
}
