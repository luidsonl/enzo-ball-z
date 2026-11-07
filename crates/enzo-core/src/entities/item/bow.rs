//! Arco: arma ativa que exige Velocidade.

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
