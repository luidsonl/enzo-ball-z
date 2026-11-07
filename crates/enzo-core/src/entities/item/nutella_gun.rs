//! Arma de Nutela: arma ativa que concede a skill Tiro de Nutela.

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
