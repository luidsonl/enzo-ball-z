//! Equipamento: tags, modificadores de atributos e slots.
//!
//! Requisitos de movimento podem exigir tags (`SWORD`, `BOW`) além de atributos.
//! O `AttributeSet` do equipamento é somado ao do personagem antes da validação.
//!
//! Itens equipáveis vivem em [`crate::entities::item`] e declaram suas próprias restrições
//! de tipo de personagem e atributos.

use super::attributes::{AttributeId, AttributeSet};
use crate::entities::item::ItemKind;

/// Bitset determinístico de tags de equipamento. Novas tags entram como novas
/// constantes; requisitos combinam tags com `contains`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EquipmentTags(u16);

impl EquipmentTags {
    pub const NONE: Self = Self(0);
    pub const SWORD: Self = Self(1 << 0);
    pub const BOW: Self = Self(1 << 1);
    pub const NUTELLA_GUN: Self = Self(1 << 2);

    pub const fn new(bits: u16) -> Self {
        Self(bits)
    }

    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// `true` se todas as tags de `required` estão presentes.
    pub fn contains(self, required: Self) -> bool {
        self.0 & required.0 == required.0
    }

    pub fn insert(&mut self, tag: Self) {
        self.0 |= tag.0;
    }

    pub fn with(mut self, tag: Self) -> Self {
        self.insert(tag);
        self
    }
}

/// Slot ocupado por um item equipado. Itens de consumo não ocupam slot.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EquipSlot {
    Weapon,
    Armor,
    Accessory,
}

impl EquipSlot {
    pub const COUNT: usize = 3;

    pub fn index(self) -> usize {
        match self {
            Self::Weapon => 0,
            Self::Armor => 1,
            Self::Accessory => 2,
        }
    }
}

/// Equipamento ativo de uma entidade, agregado a partir dos itens equipados.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Equipment {
    pub tags: EquipmentTags,
    pub modifiers: AttributeSet,
}

impl Equipment {
    pub const NONE: Self = Self {
        tags: EquipmentTags::NONE,
        modifiers: AttributeSet::ZERO,
    };

    pub fn sword() -> Self {
        Self {
            tags: EquipmentTags::SWORD,
            modifiers: AttributeSet::ZERO.with(AttributeId::MELEE, 1),
        }
    }

    pub fn bow() -> Self {
        Self {
            tags: EquipmentTags::BOW,
            modifiers: AttributeSet::ZERO.with(AttributeId::RANGED, 1),
        }
    }

    pub fn has(self, tag: EquipmentTags) -> bool {
        self.tags.contains(tag)
    }

    /// Agrega os itens equipados. A ordem é fixa por slot, então o resultado é
    /// determinístico.
    pub fn from_equipped(items: &[Option<EquippedItem>; EquipSlot::COUNT]) -> Self {
        let mut tags = EquipmentTags::NONE;
        let mut modifiers = AttributeSet::ZERO;
        for item in items.iter().flatten() {
            tags = tags.with(item.equipment.tags);
            modifiers = modifiers.merged(item.equipment.modifiers);
        }
        Self { tags, modifiers }
    }
}

/// Item instalado em um slot. Guarda a contribuição do item, não a entidade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EquippedItem {
    pub slot: EquipSlot,
    pub kind: ItemKind,
    pub equipment: Equipment,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sword_grants_melee_attribute_and_tag() {
        let sword = Equipment::sword();
        assert!(sword.has(EquipmentTags::SWORD));
        assert!(!sword.has(EquipmentTags::BOW));
        assert_eq!(sword.modifiers.get(AttributeId::MELEE), 1);
        assert_eq!(sword.modifiers.get(AttributeId::RANGED), 0);
    }

    #[test]
    fn tags_require_all_bits() {
        let tags = EquipmentTags::SWORD.with(EquipmentTags::BOW);
        assert!(tags.contains(EquipmentTags::SWORD));
        assert!(tags.contains(EquipmentTags::BOW));
        assert!(tags.contains(EquipmentTags::SWORD.with(EquipmentTags::BOW)));
        assert!(!EquipmentTags::SWORD.contains(EquipmentTags::BOW));
        assert!(EquipmentTags::NONE.is_empty());
    }

    #[test]
    fn equipped_items_aggregate_by_slot() {
        let mut items = [None; EquipSlot::COUNT];
        items[EquipSlot::Weapon.index()] = Some(EquippedItem {
            slot: EquipSlot::Weapon,
            kind: ItemKind::Sword,
            equipment: Equipment::sword(),
        });
        items[EquipSlot::Armor.index()] = Some(EquippedItem {
            slot: EquipSlot::Armor,
            kind: ItemKind::Armor,
            equipment: Equipment {
                tags: EquipmentTags::NONE,
                modifiers: AttributeSet::ZERO.with(AttributeId::HEALTH, 1),
            },
        });

        let total = Equipment::from_equipped(&items);
        assert!(total.has(EquipmentTags::SWORD));
        assert_eq!(total.modifiers.get(AttributeId::MELEE), 1);
        assert_eq!(total.modifiers.get(AttributeId::HEALTH), 1);
    }
}
