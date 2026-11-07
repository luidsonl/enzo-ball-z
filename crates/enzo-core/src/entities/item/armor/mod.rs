//! Armor: passive item that requires Health.

use crate::skills::EntityType;

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

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ICON: &str = "assets/items/armor/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/armor/world.png";
}
