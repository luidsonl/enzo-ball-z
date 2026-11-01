//! Entidade jogável **Enzo**.
//!
//! O Enzo é o personagem jogável do MVP. O core continua preparado para N
//! jogadores: cada um recebe um [`PlayerData`] com `player_id` estável, mas
//! todos usam o mesmo corpo e o mesmo conjunto inicial de movimentos.
//!
//! A construção da entidade fica aqui, junto dos números do personagem. Os
//! componentes genéricos (`PlayerData`, `Movement`, `Combat`) continuam em seus
//! próprios módulos.

use crate::attacks::Combat;
use crate::math::Vec2;
use crate::movements::Movement;

use super::attributes::{Attributes, Stats};
use super::components::{Health, TeamId};
use super::controller::Controller;
use super::entity::EntityKind;
use super::equipment::{EquipSlot, Equipment};
use super::player::{PlayerData, PlayerId};
use super::spec::EntitySpec;

/// Corpo do Enzo. O tamanho é do personagem, não do controlador.
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
            movement: Movement::player(),
            combat: Combat::player(),
            velocity: Vec2::ZERO,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::MoveId;

    #[test]
    fn enzo_spec_is_playable_with_default_moves() {
        let spec = EntitySpec::player(0, Vec2::ZERO);
        assert!(spec.controller.is_playable());
        assert_eq!(spec.team, TeamId::PLAYERS);
        assert_eq!(spec.size, ENZO_SIZE);
        assert!(spec.movement.has(MoveId::Walk));
        assert!(spec.movement.has(MoveId::Jump));
        assert!(spec.movement.has(MoveId::Dash));
        assert!(spec.combat.has(MoveId::Melee));
        assert!(spec.combat.has(MoveId::Mogar));
        assert!(spec.controller.ai().is_none());
    }
}
