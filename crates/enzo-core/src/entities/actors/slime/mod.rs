//! **Slime** enemy actors (regular and giant variants).

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

/// Slime body size and combat tuning.
pub const SLIME_SIZE: Vec2 = Vec2::new(26.0, 26.0);
pub const SLIME_XP_REWARD: f32 = 25.0;
/// Slime contact damage (rebalance target 2-4, integer). Fixed at 3 for determinism.
pub const SLIME_CONTACT_DAMAGE: f32 = 3.0;

/// Giant slime: stronger slime that splits into 3 regular slimes on death.
pub const GIANT_SLIME_SIZE: Vec2 = Vec2::new(52.0, 52.0);
pub const GIANT_SLIME_XP_REWARD: f32 = 60.0;
pub const GIANT_SLIME_CONTACT_DAMAGE: f32 = 5.0;

/// Visual/animation contract (pure data). Rendering resolves frames from this.
pub mod visual {
    pub const SHEET: &str = "assets/entities/slime/sheet.png";
    pub const ATLAS: &str = "assets/entities/slime/atlas.json";
    pub const CELL: f32 = 32.0;
    pub const CLIPS: &[(&str, &[usize], f32)] = &[
        ("walk", &[0, 1], 0.18),
        ("stretch", &[2], 0.10),
        ("hurt", &[3], 0.12),
        ("dying", &[4, 5], 0.12),
    ];
    pub const WALK_A: usize = 0;
    pub const WALK_B: usize = 1;
    pub const STRETCH: usize = 2;
    pub const HURT: usize = 3;
    pub const DYING_A: usize = 4;
    pub const DYING_B: usize = 5;
}

impl Stats {
    pub fn slime() -> Self {
        Self {
            walk_speed: 60.0,
            run_speed: 60.0,
            jump_speed: 0.0,
            strength: 0.0,
            max_health: 3.0,
            max_aura: 0.0,
            mogging_damage: 0.0,
        }
    }

    pub fn giant_slime() -> Self {
        Self {
            walk_speed: 50.0,
            run_speed: 50.0,
            jump_speed: 0.0,
            strength: 0.0,
            max_health: 9.0,
            max_aura: 0.0,
            mogging_damage: 0.0,
        }
    }
}

impl EntitySpec {
    fn slime_variant(
        pos: Vec2,
        species: EnemySpecies,
        size: Vec2,
        stats: Stats,
        xp_reward: f32,
        contact_damage: f32,
    ) -> Self {
        Self {
            kind: EntityKind::Enemy(EnemyData {
                species,
                xp_reward,
                contact_damage,
                stun: 0.0,
                xp_awarded: false,
                death_timer: 0.0,
            }),
            controller: Controller::from_ai(Ai::slime()),
            team: TeamId::ENEMIES,
            pos,
            size,
            health: Health::full(stats.max_health),
            stats,
            attributes: Attributes::ZERO,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            locomotion: Locomotion::enemy(),
            skills: Skills::enemy(),
            velocity: Vec2::ZERO,
            visual_character: None,
            visual_profile: Default::default(),
        }
    }

    pub fn slime(pos: Vec2) -> Self {
        Self::slime_variant(
            pos,
            EnemySpecies::Slime,
            SLIME_SIZE,
            Stats::slime(),
            SLIME_XP_REWARD,
            SLIME_CONTACT_DAMAGE,
        )
    }

    pub fn giant_slime(pos: Vec2) -> Self {
        Self::slime_variant(
            pos,
            EnemySpecies::GiantSlime,
            GIANT_SLIME_SIZE,
            Stats::giant_slime(),
            GIANT_SLIME_XP_REWARD,
            GIANT_SLIME_CONTACT_DAMAGE,
        )
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

    #[test]
    fn giant_slime_is_stronger_than_regular_slime() {
        let giant = EntitySpec::giant_slime(Vec2::ZERO);
        let slime = EntitySpec::slime(Vec2::ZERO);
        assert!(giant.size.x > slime.size.x && giant.size.y > slime.size.y);
        assert!(giant.stats.max_health > slime.stats.max_health);
        let giant_enemy = giant.kind.enemy().copied().unwrap();
        let slime_enemy = slime.kind.enemy().copied().unwrap();
        assert!(giant_enemy.contact_damage > slime_enemy.contact_damage);
        assert!(giant_enemy.xp_reward > slime_enemy.xp_reward);
        assert_eq!(giant_enemy.species, EnemySpecies::GiantSlime);
    }
}
