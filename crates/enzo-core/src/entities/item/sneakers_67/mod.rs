//! 67 Sneakers: themed footwear.

use crate::skills::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
use crate::entities::{AttributeId, AttributeSet, Equipment, EquipmentTags};

impl ItemData {
    pub const SNEAKERS_67: Self = Self {
        kind: ItemKind::Sneakers67,
        category: ItemCategory::Footwear,
        equipment: Equipment {
            tags: EquipmentTags::SIXSEVEN,
            modifiers: AttributeSet::ZERO.with(AttributeId::SPEED, 1),
            skill_levels: [0, 0, 0, 0, 1, 0, 0],
        },
        requirements: ItemRequirements::new(&[EntityType::Playable], AttributeSet::ZERO),
        heal: 0.0,
    };
}

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ID: &str = "sneakers_67";
    pub const DISPLAY_NAME: &str = "67 Sneakers";
    pub const VISUAL_SLOT: &str = "feet";
    pub const ICON: &str = "assets/items/sneakers_67/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/sneakers_67/world.png";
    pub const EQUIPPED_SPRITE: &str = "assets/items/sneakers_67/equipped.png";
}
