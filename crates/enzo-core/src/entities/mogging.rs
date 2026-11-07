//! Builders de projéteis/efeitos de skill.

use crate::math::Vec2;
use crate::mechanics::Locomotion;
use crate::skills::{SkillId, Skills};

use super::attributes::{Attributes, Stats};
use super::components::{Health, TeamId};
use super::controller::Controller;
use super::entity::{EntityId, EntityKind};
use super::equipment::{EquipSlot, Equipment};
use super::projectile::ProjectileData;
use super::spec::EntitySpec;

pub const MOGGING_SIZE: Vec2 = Vec2::new(24.0, 12.0);
pub const MOGGING_SPEED: f32 = 620.0;
pub const MOGGING_LIFETIME: f32 = 1.4;

impl Stats {
    /// Projétil não é ator: vida simbólica, sem movimento próprio.
    pub fn projectile() -> Self {
        Self {
            walk_speed: 0.0,
            run_speed: 0.0,
            jump_speed: 0.0,
            strength: 0.0,
            max_health: 1.0,
            max_aura: 0.0,
            mogging_damage: 0.0,
        }
    }
}

impl EntitySpec {
    pub fn projectile(
        skill: SkillId,
        owner: EntityId,
        pos: Vec2,
        facing: i8,
        damage: f32,
        team: TeamId,
        size: Vec2,
        speed: f32,
        lifetime: f32,
    ) -> Self {
        let stats = Stats::projectile();
        let direction = if facing >= 0 { 1.0 } else { -1.0 };
        Self {
            kind: EntityKind::Projectile(ProjectileData {
                owner,
                skill,
                damage,
                hits: Vec::new(),
                lifetime,
            }),
            controller: Controller::None,
            team,
            pos,
            size,
            health: Health::full(stats.max_health),
            stats,
            attributes: Attributes::ZERO,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            locomotion: Locomotion::none(),
            skills: Skills::none(),
            velocity: Vec2::new(speed * direction, 0.0),
        }
    }

    /// Compat: projétil Mogging com os números padrão.
    pub fn mogging(owner: EntityId, pos: Vec2, facing: i8, damage: f32, team: TeamId) -> Self {
        Self::projectile(
            SkillId::Mogging,
            owner,
            pos,
            facing,
            damage,
            team,
            MOGGING_SIZE,
            MOGGING_SPEED,
            MOGGING_LIFETIME,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntityStore;

    #[test]
    fn mogging_is_a_projectile_entity() {
        let mut store = EntityStore::new();
        let owner = store.spawn(EntitySpec::player(0, Vec2::ZERO));
        let id = store.spawn(EntitySpec::mogging(
            owner,
            Vec2::new(10.0, 8.0),
            1,
            20.0,
            TeamId::PLAYERS,
        ));
        let entity = store.get(id).unwrap();

        assert_eq!(entity.body.size, MOGGING_SIZE);
        assert_eq!(entity.body.vel.x, MOGGING_SPEED);
        assert_eq!(entity.projectile().unwrap().owner, owner);
        assert_eq!(entity.projectile().unwrap().skill, SkillId::Mogging);
        assert_eq!(entity.team, TeamId::PLAYERS);
        assert!(entity.controller.is_none());
        assert!(entity.locomotion.walk.intent_x == 0);
        assert!(entity.skills.loadout.is_empty());
    }
}
