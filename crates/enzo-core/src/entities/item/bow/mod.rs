//! Bow: active weapon that requires Speed.

use crate::skills::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
use crate::entities::{AttributeId, AttributeSet, Equipment, EquipmentTags};

impl ItemData {
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
}

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ICON: &str = "assets/items/bow/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/bow/world.png";
}
