//! 67 Sword: themed active weapon.

use crate::skills::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
use crate::entities::{AttributeId, AttributeSet, Equipment, EquipmentTags};

impl ItemData {
    pub const SWORD_67: Self = Self {
        kind: ItemKind::Sword67,
        category: ItemCategory::Weapon,
        equipment: Equipment {
            tags: EquipmentTags::SWORD.union(EquipmentTags::SIXSEVEN),
            modifiers: AttributeSet::ZERO
                .with(AttributeId::MELEE, 1)
                .with(AttributeId::EGO, 1),
            skill_levels: [0, 0, 0, 0, 1, 0, 0],
        },
        requirements: ItemRequirements::new(
            &[EntityType::Playable],
            AttributeSet::ZERO.with(AttributeId::STRENGTH, 1),
        ),
        heal: 0.0,
    };
}

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ID: &str = "sword_67";
    pub const DISPLAY_NAME: &str = "67 Sword";
    pub const VISUAL_SLOT: &str = "weapon";
    pub const ICON: &str = "assets/items/sword_67/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/sword_67/world.png";
    pub const EQUIPPED_SPRITE: &str = "assets/items/sword_67/equipped.png";
}
