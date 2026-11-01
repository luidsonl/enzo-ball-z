//! Entidade **slime**.
//!
//! Inimigo inicial: anda em linha reta, vira em paredes/beiradas e causa dano
//! por contato. O comportamento fica em [`crate::ai::slime`]; aqui ficam os
//! números da espécie e a construção da entidade.

use crate::ai::Ai;
use crate::attacks::Combat;
use crate::math::Vec2;
use crate::movements::Movement;

use super::attributes::{Attributes, Stats};
use super::components::{Health, TeamId};
use super::enemy::{EnemyData, EnemySpecies};
use super::entity::{Controller, EntityKind};
use super::equipment::{EquipSlot, Equipment};
use super::spec::EntitySpec;

/// Corpo da slime.
pub const SLIME_SIZE: Vec2 = Vec2::new(26.0, 26.0);
pub const SLIME_XP_REWARD: f32 = 25.0;
pub const SLIME_CONTACT_DAMAGE: f32 = 10.0;

impl Stats {
    pub fn slime() -> Self {
        Self {
            walk_speed: 60.0,
            run_speed: 60.0,
            jump_speed: 0.0,
            strength: 0.0,
            max_health: 30.0,
            max_aura: 0.0,
            mogar_damage: 0.0,
        }
    }
}

impl EnemyData {
    pub fn slime() -> Self {
        Self {
            species: EnemySpecies::Slime,
            xp_reward: SLIME_XP_REWARD,
            contact_damage: SLIME_CONTACT_DAMAGE,
            stun: 0.0,
            xp_awarded: false,
        }
    }
}

impl Default for EnemyData {
    fn default() -> Self {
        Self::slime()
    }
}

impl EntitySpec {
    /// Cria uma slime com IA de patrulha e dano por contato.
    pub fn slime(pos: Vec2) -> Self {
        let stats = Stats::slime();
        Self {
            kind: EntityKind::Enemy(EnemyData::slime()),
            controller: Controller::Ai,
            team: TeamId::ENEMIES,
            pos,
            size: SLIME_SIZE,
            health: Health::full(stats.max_health),
            stats,
            attributes: Attributes::ZERO,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            movement: Movement::enemy(),
            combat: Combat::none(),
            ai: Some(Ai::slime()),
            velocity: Vec2::new(-stats.walk_speed, 0.0),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::MoveId;

    #[test]
    fn slime_spec_has_ai_and_no_player_moves() {
        let spec = EntitySpec::slime(Vec2::ZERO);
        assert_eq!(spec.controller, Controller::Ai);
        assert_eq!(spec.team, TeamId::ENEMIES);
        assert_eq!(spec.size, SLIME_SIZE);
        assert!(spec.ai.is_some());
        assert!(spec.movement.has(MoveId::Walk));
        assert!(!spec.movement.has(MoveId::Jump));
        assert!(!spec.combat.has(MoveId::Melee));
    }

    #[test]
    fn slime_data_carries_species_numbers() {
        let slime = EnemyData::slime();
        assert_eq!(slime.species, EnemySpecies::Slime);
        assert_eq!(slime.xp_reward, SLIME_XP_REWARD);
        assert_eq!(slime.contact_damage, SLIME_CONTACT_DAMAGE);
        assert_eq!(slime.stun, 0.0);
        assert!(!slime.xp_awarded);
    }
}
