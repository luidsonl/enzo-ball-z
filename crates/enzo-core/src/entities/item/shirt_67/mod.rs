//! 67 Shirt: themed torso clothing.

use crate::skills::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
use crate::entities::{AttributeId, AttributeSet, Equipment, EquipmentTags};

impl ItemData {
    pub const SHIRT_67: Self = Self {
        kind: ItemKind::Shirt67,
        category: ItemCategory::Armor,
        equipment: Equipment {
            tags: EquipmentTags::SIXSEVEN,
            modifiers: AttributeSet::ZERO.with(AttributeId::HEALTH, 1),
            skill_levels: [0, 0, 0, 0, 1, 0, 0],
        },
        requirements: ItemRequirements::new(&[EntityType::Playable], AttributeSet::ZERO),
        heal: 0.0,
    };
}

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ID: &str = "shirt_67";
    pub const DISPLAY_NAME: &str = "67 Shirt";
    pub const VISUAL_SLOT: &str = "torso";
    pub const ICON: &str = "assets/items/shirt_67/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/shirt_67/world.png";
    pub const EQUIPPED_SPRITE: &str = "assets/items/shirt_67/equipped.png";
}
