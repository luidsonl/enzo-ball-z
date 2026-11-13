//! Especificação genérica de criação de entidades.
//!
//! Nada de montar [`Entity`] à mão nos sistemas: use um construtor concreto e
//! [`crate::entities::EntityStore::spawn`].

use crate::math::Vec2;
use crate::mechanics::Locomotion;
use crate::skills::Skills;

use super::actors::player::{PlayableCharacter, VisualProfile};
use super::base::attributes::{Attributes, Stats};
use super::base::components::{Health, TeamId};
use super::base::controller::Controller;
use super::base::entity::EntityKind;
use super::base::equipment::{EquipSlot, Equipment, EquippedItem};
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
    pub locomotion: Locomotion,
    pub skills: Skills,
    pub velocity: Vec2,
    /// Visual appearance for humanoid rendering (players and humanoid-style
    /// enemies/variants based on Enzo/Valentina).
    pub visual_character: Option<PlayableCharacter>,
    pub visual_profile: VisualProfile,
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

    pub fn with_locomotion(mut self, locomotion: Locomotion) -> Self {
        self.locomotion = locomotion;
        self
    }

    pub fn with_skills(mut self, skills: Skills) -> Self {
        self.skills = skills;
        self
    }

    pub fn with_team(mut self, team: TeamId) -> Self {
        self.team = team;
        self
    }

    /// Sets visual appearance (for humanoid characters/enemies with varied traits).
    pub fn with_visual(
        mut self,
        character: Option<PlayableCharacter>,
        profile: VisualProfile,
    ) -> Self {
        self.visual_character = character;
        self.visual_profile = profile;
        self
    }

    pub fn with_ai(mut self, ai: crate::ai::Ai) -> Self {
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

    pub fn skill_context(&self) -> crate::skills::SkillContext {
        crate::skills::SkillContext::new(
            self.kind.entity_type(),
            self.kind.level(),
            self.attributes.to_set().merged(self.equipment.modifiers),
            self.equipment.tags,
            self.equipment.skill_levels,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::skills::{EntityType, SkillId};

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
            armed.skill_context().equipment,
            ItemData::SWORD.equipment.tags
        );
        assert_eq!(armed.skill_context().entity_type, EntityType::Playable);
        assert_eq!(
            armed.skills.loadout.get(crate::input::ActionSlot::Slot1),
            Some(SkillId::Jump)
        );
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
