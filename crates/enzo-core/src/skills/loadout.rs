//! Loadout: vínculo `slot → skill` com invariante de unicidade.
//!
//! O loadout é `Copy`, serializável e determinístico (array fixo). Atribuir uma
//! skill já equipada **troca** os slots em vez de duplicar; saves corrompidos
//! passam por [`Loadout::sanitize`].

use crate::input::ActionSlot;

use super::{SkillId, SkillLock};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(transparent))]
pub struct Loadout {
    slots: [Option<SkillId>; ActionSlot::COUNT],
}

impl Loadout {
    pub const EMPTY: Self = Self {
        slots: [None; ActionSlot::COUNT],
    };

    /// Loadout inicial do Enzo: Pulo, Dash, Soco e Mogging.
    pub const fn player_default() -> Self {
        let mut slots = [None; ActionSlot::COUNT];
        slots[ActionSlot::Slot1.index()] = Some(SkillId::Jump);
        slots[ActionSlot::Slot2.index()] = Some(SkillId::Dash);
        slots[ActionSlot::Slot3.index()] = Some(SkillId::Melee);
        slots[ActionSlot::Slot4.index()] = Some(SkillId::Mogging);
        Self { slots }
    }

    pub const fn get(self, slot: ActionSlot) -> Option<SkillId> {
        self.slots[slot.index()]
    }

    pub const fn contains(self, skill: SkillId) -> bool {
        let mut index = 0;
        while index < ActionSlot::COUNT {
            if let Some(current) = self.slots[index] {
                if current as u8 == skill as u8 {
                    return true;
                }
            }
            index += 1;
        }
        false
    }

    pub const fn slot_of(self, skill: SkillId) -> Option<ActionSlot> {
        let mut index = 0;
        while index < ActionSlot::COUNT {
            if let Some(current) = self.slots[index] {
                if current as u8 == skill as u8 {
                    return ActionSlot::from_index(index);
                }
            }
            index += 1;
        }
        None
    }

    pub fn iter(self) -> impl Iterator<Item = (ActionSlot, SkillId)> {
        ActionSlot::ALL
            .into_iter()
            .filter_map(move |slot| self.get(slot).map(|skill| (slot, skill)))
    }

    pub fn is_empty(self) -> bool {
        self.slots.iter().all(Option::is_none)
    }

    /// Atribui ou troca, mantendo a unicidade por construção.
    pub fn assign(&mut self, slot: ActionSlot, skill: SkillId) -> LoadoutChange {
        let current = self.slots[slot.index()];
        if current == Some(skill) {
            return LoadoutChange::None;
        }

        if let Some(existing) = self.slot_of(skill) {
            self.slots[slot.index()] = Some(skill);
            self.slots[existing.index()] = current;
            return LoadoutChange::Swapped {
                a: slot,
                b: existing,
            };
        }

        self.slots[slot.index()] = Some(skill);
        LoadoutChange::Assigned {
            slot,
            skill,
            previous: current,
        }
    }

    pub fn clear(&mut self, slot: ActionSlot) -> Option<SkillId> {
        self.slots[slot.index()].take()
    }

    /// Remove duplicatas mantendo a primeira ocorrência na ordem dos slots.
    /// Devolve `true` se algo foi alterado.
    pub fn sanitize(&mut self) -> bool {
        let mut seen = [false; SkillId::COUNT];
        let mut changed = false;
        for slot in ActionSlot::ALL {
            let Some(skill) = self.slots[slot.index()] else {
                continue;
            };
            let index = skill.index();
            if seen[index] {
                self.slots[slot.index()] = None;
                changed = true;
            } else {
                seen[index] = true;
            }
        }
        changed
    }
}

/// Resultado de uma operação de loadout, usado para evento e toast.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadoutChange {
    None,
    Assigned {
        slot: ActionSlot,
        skill: SkillId,
        previous: Option<SkillId>,
    },
    Swapped {
        a: ActionSlot,
        b: ActionSlot,
    },
    Cleared {
        slot: ActionSlot,
        skill: SkillId,
    },
}

/// Pedido de troca de loadout. Não entra em `PlayerCommand`: é meta-estado
/// esparso aplicado com o jogo pausado.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LoadoutRequest {
    Assign { slot: ActionSlot, skill: SkillId },
    Clear { slot: ActionSlot },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadoutError {
    NoPlayer,
    EmptySlot,
    Unavailable { skill: SkillId, lock: SkillLock },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_loadout_is_unique_and_complete() {
        let loadout = Loadout::player_default();
        assert_eq!(loadout.get(ActionSlot::Slot1), Some(SkillId::Jump));
        assert_eq!(loadout.get(ActionSlot::Slot2), Some(SkillId::Dash));
        assert_eq!(loadout.get(ActionSlot::Slot3), Some(SkillId::Melee));
        assert_eq!(loadout.get(ActionSlot::Slot4), Some(SkillId::Mogging));
        assert!(loadout.contains(SkillId::Mogging));
        assert_eq!(loadout.slot_of(SkillId::Mogging), Some(ActionSlot::Slot4));
        assert_eq!(loadout.iter().count(), 4);
        assert!(!loadout.is_empty());
    }

    #[test]
    fn assigning_unslotted_skill_replaces_slot() {
        let mut loadout = Loadout::player_default();
        let change = loadout.assign(ActionSlot::Slot1, SkillId::Fly);
        assert_eq!(
            change,
            LoadoutChange::Assigned {
                slot: ActionSlot::Slot1,
                skill: SkillId::Fly,
                previous: Some(SkillId::Jump),
            }
        );
        assert_eq!(loadout.get(ActionSlot::Slot1), Some(SkillId::Fly));
        assert!(!loadout.contains(SkillId::Jump));
    }

    #[test]
    fn assigning_equipped_skill_swaps_slots() {
        let mut loadout = Loadout::player_default();
        let change = loadout.assign(ActionSlot::Slot1, SkillId::Mogging);
        assert_eq!(
            change,
            LoadoutChange::Swapped {
                a: ActionSlot::Slot1,
                b: ActionSlot::Slot4,
            }
        );
        assert_eq!(loadout.get(ActionSlot::Slot1), Some(SkillId::Mogging));
        assert_eq!(loadout.get(ActionSlot::Slot4), Some(SkillId::Jump));
        assert_eq!(loadout.iter().count(), 4);
    }

    #[test]
    fn assigning_to_the_same_slot_is_a_no_op() {
        let mut loadout = Loadout::player_default();
        assert_eq!(
            loadout.assign(ActionSlot::Slot2, SkillId::Dash),
            LoadoutChange::None
        );
        assert_eq!(loadout, Loadout::player_default());
    }

    #[test]
    fn assigning_equipped_skill_to_empty_slot_moves_it() {
        let mut loadout = Loadout::player_default();
        loadout.clear(ActionSlot::Slot2);
        let change = loadout.assign(ActionSlot::Slot2, SkillId::Jump);
        assert_eq!(
            change,
            LoadoutChange::Swapped {
                a: ActionSlot::Slot2,
                b: ActionSlot::Slot1,
            }
        );
        assert_eq!(loadout.get(ActionSlot::Slot1), None);
        assert_eq!(loadout.get(ActionSlot::Slot2), Some(SkillId::Jump));
    }

    #[test]
    fn clear_returns_skill_and_empties_slot() {
        let mut loadout = Loadout::player_default();
        assert_eq!(loadout.clear(ActionSlot::Slot3), Some(SkillId::Melee));
        assert_eq!(loadout.clear(ActionSlot::Slot3), None);
        assert_eq!(loadout.get(ActionSlot::Slot3), None);
    }

    #[test]
    fn sanitize_keeps_first_occurrence() {
        let mut loadout = Loadout {
            slots: [
                Some(SkillId::Jump),
                Some(SkillId::Jump),
                Some(SkillId::Dash),
                Some(SkillId::Dash),
            ],
        };
        assert!(loadout.sanitize());
        assert_eq!(loadout.get(ActionSlot::Slot1), Some(SkillId::Jump));
        assert_eq!(loadout.get(ActionSlot::Slot2), None);
        assert_eq!(loadout.get(ActionSlot::Slot3), Some(SkillId::Dash));
        assert_eq!(loadout.get(ActionSlot::Slot4), None);
        assert!(!loadout.sanitize());
    }

    #[test]
    fn efficient_contains_and_slot_of_match_iteration() {
        let loadout = Loadout::player_default();
        for skill in SkillId::ALL {
            let expected = loadout.iter().find(|(_, equipped)| *equipped == skill);
            assert_eq!(loadout.slot_of(skill), expected.map(|(slot, _)| slot));
            assert_eq!(loadout.contains(skill), expected.is_some());
        }
    }
}
