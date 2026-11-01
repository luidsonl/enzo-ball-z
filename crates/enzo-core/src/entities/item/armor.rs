//! Armadura: item passivo que exige Vida.

use crate::moves::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
use crate::entities::{AttributeId, AttributeSet, Equipment, EquipmentTags};

impl ItemData {
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
}
