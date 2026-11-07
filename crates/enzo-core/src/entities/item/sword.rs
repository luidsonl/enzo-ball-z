//! Espada: arma ativa que exige Força.

use crate::skills::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
use crate::entities::{AttributeId, AttributeSet, Equipment, EquipmentTags};

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
}
