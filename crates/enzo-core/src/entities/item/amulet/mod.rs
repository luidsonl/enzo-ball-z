//! Amulet: passive accessory that requires Aura.

use crate::skills::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
use crate::entities::{AttributeId, AttributeSet, Equipment, EquipmentTags};

impl ItemData {
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
}

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ICON: &str = "assets/items/amulet/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/amulet/world.png";
}
