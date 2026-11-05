//! Especificação genérica de criação de entidades.
//!
//! Nada de montar [`Entity`] à mão nos sistemas: use um construtor concreto
//! (`EntitySpec::player`, `slime`, `mogar`, `item`) e
//! [`crate::entities::EntityStore::spawn`].
//!
//! Os construtores concretos vivem junto da entidade correspondente
//! (`enzo.rs`, `slime.rs`, `mogar.rs`, `item.rs`). Este módulo guarda só o
//! formato comum e os builders que qualquer entidade pode usar.

use crate::ai::Ai;
use crate::attacks::Combat;
use crate::math::Vec2;
use crate::movements::Movement;
use crate::moves::{AbilityContext, AssignError, MoveCategory, MoveId};

use super::attributes::{Attributes, Stats};
use super::components::{Health, TeamId};
use super::controller::Controller;
use super::entity::EntityKind;
use super::equipment::{EquipSlot, Equipment, EquippedItem};
use super::item::{EquipError, ItemData};

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
    pub velocity: Vec2,
}

impl EntitySpec {
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

    /// Anexa um controlador de IA, substituindo o controlador atual.
    pub fn with_ai(mut self, ai: Ai) -> Self {
        self.controller = Controller::from_ai(ai);
        self
    }

    /// Equipa um item na criação, validando tipo de personagem e atributos.
    pub fn with_item(mut self, item: &ItemData) -> Result<Self, EquipError> {
        let slot = item.slot().ok_or(EquipError::NotEquippable)?;
        item.requirements
            .allows(self.kind.entity_type(), self.attributes.to_set())?;
        self.equipped[slot.index()] = Some(EquippedItem {
            slot,
            kind: item.kind,
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
        assert!(armed.equipment.has(crate::entities::EquipmentTags::SWORD));
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
