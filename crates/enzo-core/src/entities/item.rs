//! Itens: entidades que podem ser equipadas ou consumidas.
//!
//! Um item é uma entidade não-jogável (`EntityKind::Item`) e não é um ator.
//! Antes de equipar, o core valida o tipo de personagem e os atributos do
//! personagem contra [`ItemRequirements`].
//!
//! - **Ativos:** armas e poções. Armas ocupam o slot de arma; poções são
//!   consumidas e não ocupam slot.
//! - **Passivos:** armaduras e acessórios. Ficam equipados e contribuem com
//!   tags/modificadores enquanto durar o equipamento.

use crate::math::Vec2;
use crate::moves::EntityType;

use super::attributes::{AttributeId, AttributeSet};
use super::equipment::{EquipSlot, Equipment, EquipmentTags};

pub const ITEM_SIZE: Vec2 = Vec2::new(20.0, 20.0);

/// Como o item é usado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ItemUsage {
    Active,
    Passive,
}

/// Categoria do item. Define o slot (ou consumo) e o uso.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ItemCategory {
    Weapon,
    Potion,
    Armor,
    Accessory,
}

impl ItemCategory {
    pub fn usage(self) -> ItemUsage {
        match self {
            Self::Weapon | Self::Potion => ItemUsage::Active,
            Self::Armor | Self::Accessory => ItemUsage::Passive,
        }
    }

    pub fn slot(self) -> Option<EquipSlot> {
        match self {
            Self::Weapon => Some(EquipSlot::Weapon),
            Self::Armor => Some(EquipSlot::Armor),
            Self::Accessory => Some(EquipSlot::Accessory),
            Self::Potion => None,
        }
    }

    pub fn is_consumable(self) -> bool {
        matches!(self, Self::Potion)
    }
}

/// Identidade extensível de um item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ItemKind {
    Sword,
    Bow,
    Armor,
    Amulet,
    Potion,
}

/// Restrições para equipar/usar um item. Só atributos do personagem contam;
/// equipamento já vestido não ajuda a cumprir o requisito de outro item.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ItemRequirements {
    pub entity_types: &'static [EntityType],
    pub min_attributes: AttributeSet,
}

impl ItemRequirements {
    pub const fn new(entity_types: &'static [EntityType], min_attributes: AttributeSet) -> Self {
        Self {
            entity_types,
            min_attributes,
        }
    }

    pub fn allows(
        self,
        entity_type: EntityType,
        attributes: AttributeSet,
    ) -> Result<(), EquipError> {
        if !self.entity_types.contains(&entity_type) {
            return Err(EquipError::WrongEntityType {
                required: self.entity_types,
                current: entity_type,
            });
        }
        if !attributes.meets_requirements(self.min_attributes) {
            return Err(EquipError::AttributesTooLow {
                required: self.min_attributes,
                current: attributes,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EquipError {
    WrongEntityType {
        required: &'static [EntityType],
        current: EntityType,
    },
    AttributesTooLow {
        required: AttributeSet,
        current: AttributeSet,
    },
    NotEquippable,
}

/// Dados de um item. Itens não têm movimento, combate, IA ou vida relevante.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ItemData {
    pub kind: ItemKind,
    pub category: ItemCategory,
    /// Contribuição enquanto equipado. Poções deixam em [`Equipment::NONE`].
    pub equipment: Equipment,
    pub requirements: ItemRequirements,
    /// Cura aplicada ao consumir. Zero para itens não consumíveis.
    pub heal: f32,
}

impl ItemData {
    pub const SWORD: Self = Self {
        kind: ItemKind::Sword,
        category: ItemCategory::Weapon,
        equipment: Equipment {
            tags: EquipmentTags::SWORD,
            modifiers: AttributeSet::ZERO.with(AttributeId::MELEE, 1),
        },
        requirements: ItemRequirements::new(
            &[EntityType::Playable],
            AttributeSet::ZERO.with(AttributeId::STRENGTH, 2),
        ),
        heal: 0.0,
    };

    pub const BOW: Self = Self {
        kind: ItemKind::Bow,
        category: ItemCategory::Weapon,
        equipment: Equipment {
            tags: EquipmentTags::BOW,
            modifiers: AttributeSet::ZERO.with(AttributeId::RANGED, 1),
        },
        requirements: ItemRequirements::new(
            &[EntityType::Playable],
            AttributeSet::ZERO.with(AttributeId::SPEED, 2),
        ),
        heal: 0.0,
    };

    pub const ARMOR: Self = Self {
        kind: ItemKind::Armor,
        category: ItemCategory::Armor,
        equipment: Equipment {
            tags: EquipmentTags::NONE,
            modifiers: AttributeSet::ZERO.with(AttributeId::HEALTH, 1),
        },
        requirements: ItemRequirements::new(
            &[EntityType::Playable],
            AttributeSet::ZERO.with(AttributeId::HEALTH, 2),
        ),
        heal: 0.0,
    };

    pub const AMULET: Self = Self {
        kind: ItemKind::Amulet,
        category: ItemCategory::Accessory,
        equipment: Equipment {
            tags: EquipmentTags::NONE,
            modifiers: AttributeSet::ZERO.with(AttributeId::AURA, 1),
        },
        requirements: ItemRequirements::new(
            &[EntityType::Playable],
            AttributeSet::ZERO.with(AttributeId::AURA, 2),
        ),
        heal: 0.0,
    };

    pub const POTION: Self = Self {
        kind: ItemKind::Potion,
        category: ItemCategory::Potion,
        equipment: Equipment::NONE,
        requirements: ItemRequirements::new(
            &[EntityType::Playable],
            AttributeSet::ZERO.with(AttributeId::AURA, 1),
        ),
        heal: 30.0,
    };

    pub fn usage(self) -> ItemUsage {
        self.category.usage()
    }

    pub fn slot(self) -> Option<EquipSlot> {
        self.category.slot()
    }

    pub fn is_consumable(self) -> bool {
        self.category.is_consumable()
    }

    /// Valida as restrições de tipo de personagem e atributos do personagem.
    pub fn can_be_equipped_by(
        self,
        entity_type: EntityType,
        character_attributes: AttributeSet,
    ) -> Result<(), EquipError> {
        if self.slot().is_none() {
            return Err(EquipError::NotEquippable);
        }
        self.requirements.allows(entity_type, character_attributes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strong() -> AttributeSet {
        AttributeSet::ZERO.with(AttributeId::STRENGTH, 2)
    }

    #[test]
    fn sword_is_active_weapon_with_requirements() {
        let sword = ItemData::SWORD;
        assert_eq!(sword.usage(), ItemUsage::Active);
        assert_eq!(sword.slot(), Some(EquipSlot::Weapon));
        assert!(!sword.is_consumable());
        assert_eq!(
            sword.can_be_equipped_by(EntityType::Playable, AttributeSet::ZERO),
            Err(EquipError::AttributesTooLow {
                required: strong(),
                current: AttributeSet::ZERO,
            })
        );
        assert!(sword
            .can_be_equipped_by(EntityType::Playable, strong())
            .is_ok());
    }

    #[test]
    fn armor_and_amulet_are_passive() {
        assert_eq!(ItemData::ARMOR.usage(), ItemUsage::Passive);
        assert_eq!(ItemData::ARMOR.slot(), Some(EquipSlot::Armor));
        assert_eq!(ItemData::AMULET.usage(), ItemUsage::Passive);
        assert_eq!(ItemData::AMULET.slot(), Some(EquipSlot::Accessory));
    }

    #[test]
    fn potion_is_consumable_and_not_equippable() {
        let potion = ItemData::POTION;
        assert!(potion.is_consumable());
        assert_eq!(potion.slot(), None);
        assert_eq!(
            potion.can_be_equipped_by(
                EntityType::Playable,
                AttributeSet::ZERO.with(AttributeId::AURA, 1),
            ),
            Err(EquipError::NotEquippable)
        );
    }

    #[test]
    fn enemies_cannot_equip_playable_items() {
        assert!(matches!(
            ItemData::SWORD.can_be_equipped_by(EntityType::Enemy, strong()),
            Err(EquipError::WrongEntityType {
                current: EntityType::Enemy,
                ..
            })
        ));
    }
}
