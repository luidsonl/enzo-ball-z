//! Poção: item ativo de consumo que restaura vida.

use crate::moves::EntityType;

use super::{ItemCategory, ItemData, ItemKind, ItemRequirements};
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
        heal: 30.0,
    };
}
