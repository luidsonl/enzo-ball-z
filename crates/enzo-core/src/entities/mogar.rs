//! Entidade **Mogar**: projétil do especial do Enzo.
//!
//! O projétil atravessa inimigos, não acerta o mesmo alvo duas vezes e morre ao
//! colidir com tile sólido ou expirar. A lógica de disparo/colisão fica em
//! [`crate::attacks::mogar`]; aqui ficam os números e a construção da entidade.

use crate::math::Vec2;

use super::attributes::Stats;
use super::components::{Health, TeamId};
use super::entity::{Controller, EntityId, EntityKind};
use super::equipment::{EquipSlot, Equipment};
use super::projectile::ProjectileData;
use super::spec::EntitySpec;

pub const MOGAR_SIZE: Vec2 = Vec2::new(24.0, 12.0);
pub const MOGAR_SPEED: f32 = 620.0;
pub const MOGAR_LIFETIME: f32 = 1.4;

impl Stats {
    /// Projétil não é ator: vida simbólica para reutilizar `Health`, sem
    /// movimento próprio.
    pub fn projectile() -> Self {
        Self {
            walk_speed: 0.0,
            run_speed: 0.0,
            jump_speed: 0.0,
            strength: 0.0,
            max_health: 1.0,
            max_aura: 0.0,
            mogar_damage: 0.0,
        }
    }
}

impl EntitySpec {
    pub fn mogar(owner: EntityId, pos: Vec2, facing: i8, damage: f32, team: TeamId) -> Self {
        let stats = Stats::projectile();
        Self {
            kind: EntityKind::Projectile(ProjectileData {
                owner,
                damage,
                hits: Vec::new(),
                lifetime: MOGAR_LIFETIME,
            }),
            controller: Controller::None,
            team,
            pos,
            size: MOGAR_SIZE,
            health: Health::full(stats.max_health),
            stats,
            attributes: super::attributes::Attributes::ZERO,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            movement: crate::movements::Movement::none(),
            combat: crate::attacks::Combat::none(),
            ai: None,
            velocity: Vec2::new(if facing >= 0 { 1.0 } else { -1.0 } * MOGAR_SPEED, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntityId;

    #[test]
    fn mogar_spec_is_a_projectile_with_owner_team_and_speed() {
        let spec = EntitySpec::mogar(EntityId(0), Vec2::ZERO, 1, 20.0, TeamId::PLAYERS);
        assert_eq!(spec.controller, Controller::None);
        assert_eq!(spec.team, TeamId::PLAYERS);
        assert_eq!(spec.size, MOGAR_SIZE);
        assert_eq!(spec.velocity.x, MOGAR_SPEED);
        assert!(spec.movement.moves.is_empty());
        assert!(spec.combat.moves.is_empty());
        assert!(spec.ai.is_none());
        match spec.kind {
            EntityKind::Projectile(data) => {
                assert_eq!(data.owner, EntityId(0));
                assert_eq!(data.damage, 20.0);
                assert_eq!(data.lifetime, MOGAR_LIFETIME);
            }
            _ => panic!("Mogar deve ser um projétil"),
        }
    }
}
