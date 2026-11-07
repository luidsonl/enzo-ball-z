//! Nutella Gun: active weapon that grants the Nutella Shot skill.

use crate::skills::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
use crate::entities::{AttributeSet, Equipment, EquipmentTags};

impl ItemData {
    pub const NUTELLA_GUN: Self = Self {
        kind: ItemKind::NutellaGun,
        category: ItemCategory::Weapon,
        equipment: Equipment {
            tags: EquipmentTags::NUTELLA_GUN,
            modifiers: AttributeSet::ZERO,
        },
        requirements: ItemRequirements::new(&[EntityType::Playable], AttributeSet::ZERO),
        heal: 0.0,
    };
}

/// Visual/animation metadata co-located with the item module.
pub mod visual {
    pub const ICON: &str = "assets/items/nutella_gun/icon.png";
    pub const WORLD_SPRITE: &str = "assets/items/nutella_gun/world.png";
    pub const GRANTS_SKILL: &str = "nutella_shot";
}
