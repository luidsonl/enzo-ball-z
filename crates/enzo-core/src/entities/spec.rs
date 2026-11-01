//! Especificação de criação de entidades.
//!
//! Nada de montar [`Entity`] à mão nos sistemas: use um construtor daqui e
//! [`crate::entities::EntityStore::spawn`]. Os construtores já ligam time,
//! controlador, movimentos, IA e equipamento iniciais.

use crate::ai::Ai;
use crate::attacks::Combat;
use crate::math::Vec2;
use crate::movements::Movement;
use crate::moves::{AbilityContext, AssignError, MoveCategory, MoveId};

use super::attributes::{Attributes, Stats};
use super::components::{Health, TeamId};
use super::enemy::{EnemyData, ENEMY_SIZE};
use super::entity::{Controller, EntityId, EntityKind};
use super::equipment::{EquipSlot, Equipment, EquippedItem};
use super::item::{EquipError, ItemData, ITEM_SIZE};
use super::player::{PlayerData, PlayerId, PLAYER_SIZE};
use super::projectile::{ProjectileData, MOGAR_LIFETIME, MOGAR_SIZE, MOGAR_SPEED};

/// Especificação de criação. Use os construtores; não monte `Entity` à mão.
#[derive(Clone, Debug, PartialEq)]
pub struct EntitySpec {
    pub kind: EntityKind,
    pub controller: Controller,
    pub team: TeamId,
    pub pos: Vec2,
    pub size: Vec2,
    pub health: Health,
    pub stats: Stats,
    pub attributes: Attributes,
    pub equipment: Equipment,
    pub equipped: [Option<EquippedItem>; EquipSlot::COUNT],
    pub movement: Movement,
    pub combat: Combat,
    pub ai: Option<Ai>,
    pub velocity: Vec2,
}

impl EntitySpec {
    pub fn player(player_id: PlayerId, pos: Vec2) -> Self {
        let attributes = Attributes::default();
        let stats = Stats::player(attributes);
        Self {
            kind: EntityKind::Player(PlayerData::new(player_id, pos, stats)),
            controller: Controller::Player,
            team: TeamId::PLAYERS,
            pos,
            size: PLAYER_SIZE,
            health: Health::full(stats.max_health),
            stats,
            attributes,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            movement: Movement::player(),
            combat: Combat::player(),
            ai: None,
            velocity: Vec2::ZERO,
        }
    }

    /// Inimigo inicial: slime que patrulha e causa dano por contato.
    pub fn slime(pos: Vec2) -> Self {
        let stats = Stats::slime();
        Self {
            kind: EntityKind::Enemy(EnemyData::slime()),
            controller: Controller::Ai,
            team: TeamId::ENEMIES,
            pos,
            size: ENEMY_SIZE,
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
            attributes: Attributes::ZERO,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            movement: Movement::none(),
            combat: Combat::none(),
            ai: None,
            velocity: Vec2::new(if facing >= 0 { 1.0 } else { -1.0 } * MOGAR_SPEED, 0.0),
        }
    }

    /// Item no mundo. Não é ator: sem movimento, combate ou IA.
    pub fn item(item: ItemData, pos: Vec2) -> Self {
        let stats = Stats::item();
        Self {
            kind: EntityKind::Item(item),
            controller: Controller::None,
            team: TeamId::NEUTRAL,
            pos,
            size: ITEM_SIZE,
            health: Health::full(stats.max_health),
            stats,
            attributes: Attributes::ZERO,
            equipment: Equipment::NONE,
            equipped: [None; EquipSlot::COUNT],
            movement: Movement::none(),
            combat: Combat::none(),
            ai: None,
            velocity: Vec2::ZERO,
        }
    }

    pub fn with_health(mut self, current: f32) -> Self {
        self.health.current = current.clamp(0.0, self.stats.max_health);
        self
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }

    pub fn with_movement(mut self, movement: Movement) -> Self {
        self.movement = movement;
        self
    }

    pub fn with_combat(mut self, combat: Combat) -> Self {
        self.combat = combat;
        self
    }

    pub fn with_team(mut self, team: TeamId) -> Self {
        self.team = team;
        self
    }

    pub fn with_ai(mut self, ai: Ai) -> Self {
        self.ai = Some(ai);
        self
    }

    /// Equipa um item na criação, validando tipo de personagem e atributos.
    pub fn with_item(mut self, item: &ItemData) -> Result<Self, EquipError> {
        let slot = item.slot().ok_or(EquipError::NotEquippable)?;
        item.requirements
            .allows(self.kind.entity_type(), self.attributes.to_set())?;
        self.equipped[slot.index()] = Some(EquippedItem {
            slot,
            equipment: item.equipment,
        });
        self.equipment = Equipment::from_equipped(&self.equipped);
        Ok(self)
    }

    pub fn ability_context(&self) -> AbilityContext {
        AbilityContext::new(
            self.kind.entity_type(),
            self.kind.level(),
            self.attributes.to_set().merged(self.equipment.modifiers),
            self.equipment.tags,
        )
    }

    /// Atribui um movimento ao spec se a entidade cumprir os requisitos.
    pub fn with_move(mut self, id: MoveId) -> Result<Self, AssignError> {
        let context = self.ability_context();
        match id.category() {
            MoveCategory::Movement => self.movement.try_assign(id, context)?,
            MoveCategory::Attack => self.combat.try_assign(id, context)?,
        }
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::moves::EntityType;

    #[test]
    fn player_spec_has_playable_controller_and_moves() {
        let spec = EntitySpec::player(0, Vec2::ZERO);
        assert_eq!(spec.controller, Controller::Player);
        assert_eq!(spec.team, TeamId::PLAYERS);
        assert!(spec.movement.has(MoveId::Walk));
        assert!(spec.combat.has(MoveId::Mogar));
        assert!(spec.ai.is_none());
    }

    #[test]
    fn slime_spec_has_ai_and_no_player_moves() {
        let spec = EntitySpec::slime(Vec2::ZERO);
        assert_eq!(spec.controller, Controller::Ai);
        assert_eq!(spec.team, TeamId::ENEMIES);
        assert!(spec.ai.is_some());
        assert!(spec.movement.has(MoveId::Walk));
        assert!(!spec.movement.has(MoveId::Jump));
        assert!(!spec.combat.has(MoveId::Melee));
    }

    #[test]
    fn item_spec_is_neutral_and_not_an_actor() {
        let spec = EntitySpec::item(ItemData::SWORD, Vec2::ZERO);
        assert_eq!(spec.controller, Controller::None);
        assert_eq!(spec.team, TeamId::NEUTRAL);
        assert!(spec.movement.moves.is_empty());
        assert!(spec.combat.moves.is_empty());
        assert!(spec.ai.is_none());
    }

    #[test]
    fn spec_rejects_move_without_requirements() {
        let player = EntitySpec::player(0, Vec2::ZERO);
        assert!(matches!(
            player.with_move(MoveId::Fly),
            Err(AssignError::LevelTooLow {
                required: 5,
                current: 1,
            })
        ));

        let enemy = EntitySpec::slime(Vec2::ZERO);
        assert!(matches!(
            enemy.with_move(MoveId::Jump),
            Err(AssignError::WrongEntityType { .. })
        ));
    }

    #[test]
    fn item_requirements_are_validated_on_creation() {
        let player = EntitySpec::player(0, Vec2::ZERO);
        assert!(matches!(
            player.with_item(&ItemData::SWORD),
            Err(EquipError::AttributesTooLow { .. })
        ));

        let mut strong = EntitySpec::player(0, Vec2::ZERO);
        strong.attributes.strength = 2;
        let armed = strong.with_item(&ItemData::SWORD).unwrap();
        assert!(armed
            .equipment
            .has(super::super::equipment::EquipmentTags::SWORD));
        assert_eq!(
            armed.ability_context().equipment,
            ItemData::SWORD.equipment.tags
        );
        assert_eq!(armed.ability_context().entity_type, EntityType::Playable);
    }

    #[test]
    fn enemy_cannot_equip_playable_item() {
        let mut slime = EntitySpec::slime(Vec2::ZERO);
        slime.attributes.strength = 99;
        assert!(matches!(
            slime.with_item(&ItemData::SWORD),
            Err(EquipError::WrongEntityType { .. })
        ));
    }
}
