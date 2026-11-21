//! Enzotella: instant-use consumable variants (chocolate pot).
//! Big (64x64) restores 100% HP, medium (32x32) restores 50%, small (16x16) restores 25%.
//! Not consumed when the player's HP is already full. Sprites are reused.

use crate::math::Vec2;
use crate::skills::EntityType;

use super::{
    ItemCategory, ItemConsumableEffect, ItemData, ItemKind, ItemPickup, ItemRequirements,
    ItemWorldPhysics,
};
use crate::entities::{AttributeSet, Equipment};

impl ItemData {
    pub const ENZOTELLA_BIG: Self = Self {
        kind: ItemKind::EnzotellaBig,
        category: ItemCategory::Potion,
        equipment: Equipment::NONE,
        requirements: ItemRequirements::new(&[EntityType::Playable], AttributeSet::ZERO),
        consumable_effect: ItemConsumableEffect::HealFractionOfMaxHealth(1.0),
        world_physics: ItemWorldPhysics::FLOATING.with_size(Vec2::new(64.0, 64.0)),
        pickup: ItemPickup::TOUCH,
    };

    pub const ENZOTELLA_MEDIUM: Self = Self {
        kind: ItemKind::EnzotellaMedium,
        category: ItemCategory::Potion,
        equipment: Equipment::NONE,
        requirements: ItemRequirements::new(&[EntityType::Playable], AttributeSet::ZERO),
        consumable_effect: ItemConsumableEffect::HealFractionOfMaxHealth(0.5),
        world_physics: ItemWorldPhysics::FLOATING.with_size(Vec2::new(32.0, 32.0)),
        pickup: ItemPickup::TOUCH,
    };

    pub const ENZOTELLA_SMALL: Self = Self {
        kind: ItemKind::EnzotellaSmall,
        category: ItemCategory::Potion,
        equipment: Equipment::NONE,
        requirements: ItemRequirements::new(&[EntityType::Playable], AttributeSet::ZERO),
        consumable_effect: ItemConsumableEffect::HealFractionOfMaxHealth(0.25),
        world_physics: ItemWorldPhysics::FLOATING.with_size(Vec2::new(16.0, 16.0)),
        pickup: ItemPickup::TOUCH,
    };
}

/// Visual/animation metadata co-located with the item module (shared/reused sprites).
pub mod visual {
    pub const ICON: &str = "assets/items/enzotella/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/enzotella/world.png";
    pub const EQUIPPED_SPRITE: &str = "assets/items/enzotella/equipped.png";
}
