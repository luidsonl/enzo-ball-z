//! Sword: active weapon that requires Strength.

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
            skill_levels: crate::skills::NO_SKILL_LEVELS,
        },
        requirements: ItemRequirements::new(
            &[EntityType::Playable],
            AttributeSet::ZERO.with(AttributeId::STRENGTH, 2),
        ),
        heal: 0.0,
    };
}

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ICON: &str = "assets/items/sword/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/sword/world.png";
}
