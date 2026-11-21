//! Potion: consumable active item that restores health.

use crate::skills::EntityType;

use super::{
    ItemCategory, ItemConsumableEffect, ItemData, ItemKind, ItemPickup, ItemRequirements,
    ItemWorldPhysics,
};
use crate::entities::{AttributeId, AttributeSet, Equipment};

impl ItemData {
    pub const POTION: Self = Self {
        kind: ItemKind::Potion,
        category: ItemCategory::Potion,
        equipment: Equipment::NONE,
        requirements: ItemRequirements::new(
            &[EntityType::Playable],
            AttributeSet::ZERO.with(AttributeId::AURA, 1),
        ),
        consumable_effect: ItemConsumableEffect::HealFixed(3.0),
        world_physics: ItemWorldPhysics::FLOATING,
        pickup: ItemPickup::TOUCH,
    };
}

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ICON: &str = "assets/items/potion/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/potion/world.png";
    pub const BOB_AMPLITUDE: f32 = 2.0;
}
