//! DOM/HTML UI bridge (FFI transport layer).
//!
//! The HTML/JS DOM layer interacts with Rust only through the exported functions
//! in this module. Semantic UI actions are queued and consumed by the app/UI
//! controller; virtual input actions are merged into gameplay input. This module
//! must not contain game rules.

use std::cell::RefCell;
use std::collections::VecDeque;

use enzo_core::input::InputAction;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VirtualActionState {
    pub pressed: bool,
    pub held: bool,
}

/// Semantic UI actions emitted by DOM. The controller/app owns behavior.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UiAction {
    MenuBack,
    MenuOpenSaves,
    MenuOpenOptions,
    MenuStart { slot: u8, new_game: bool },
    MenuResume,
    MenuRequestDelete { slot: u8 },
    MenuCancelDelete,
    MenuConfirmDelete,
    SetLanguage { code: u8 }, // 0=pt-BR, 1=en
    SetMobileControls { mode: u8 },
    SetInterfaceScale { percent: u16 },
    HudAllocate { attribute: u8 },
    HudConfirmLevelUp,
    HudRestart,
    HudOpenMenu,
    HudOpenSkills,
    LoadoutAssign { slot: u8, skill: u8 },
    LoadoutClear { slot: u8 },
    EquipInventoryItem { item: u8 },
    UnequipSlot { slot: u8 },
    SetPlayableCharacter { character: u8 },
    SetVisualProfileField { field: u8, value: u8 },
}

impl UiAction {
    /// Parse the generic `enzo_ui_action` FFI payload.
    pub fn from_ffi(code: u8, a: u8, b: u8, _c: u8) -> Option<Self> {
        Some(match code {
            1 => UiAction::MenuBack,
            2 => UiAction::MenuOpenSaves,
            3 => UiAction::MenuOpenOptions,
            4 => UiAction::MenuStart {
                slot: a,
                new_game: b != 0,
            },
            5 => UiAction::MenuResume,
            6 => UiAction::MenuRequestDelete { slot: a },
            7 => UiAction::MenuCancelDelete,
            8 => UiAction::MenuConfirmDelete,
            9 => UiAction::SetLanguage { code: a },
            10 => UiAction::SetMobileControls { mode: a },
            11 => UiAction::SetInterfaceScale {
                percent: (a as u16) * 256 + b as u16,
            },
            12 => UiAction::HudAllocate { attribute: a },
            13 => UiAction::HudConfirmLevelUp,
            14 => UiAction::HudRestart,
            15 => UiAction::HudOpenMenu,
            16 => UiAction::HudOpenSkills,
            19 => UiAction::EquipInventoryItem { item: a },
            20 => UiAction::UnequipSlot { slot: a },
            21 => UiAction::SetPlayableCharacter { character: a },
            22 => UiAction::SetVisualProfileField { field: a, value: b },
            _ => return None,
        })
    }
}

#[derive(Clone, Debug, Default)]
pub struct BridgeState {
    pub actions: [VirtualActionState; InputAction::COUNT],
    pub ui_state_json: String,
    pub pending_ui_actions: VecDeque<UiAction>,
    /// 0=off, 1=on, 2=auto (default).
    pub mobile_controls_mode: u8,
}

impl BridgeState {
    fn new() -> Self {
        Self {
            mobile_controls_mode: 2,
            ..Self::default()
        }
    }
}

thread_local! {
    static BRIDGE: RefCell<BridgeState> = RefCell::new(BridgeState::new());
}

pub fn with_bridge<R>(f: impl FnOnce(&BridgeState) -> R) -> R {
    BRIDGE.with(|b| f(&b.borrow()))
}

pub fn with_bridge_mut<R>(f: impl FnOnce(&mut BridgeState) -> R) -> R {
    BRIDGE.with(|b| f(&mut b.borrow_mut()))
}

pub fn action_down(code: u8) {
    if let Some(action) = InputAction::from_code(code) {
        with_bridge_mut(|b| {
            let s = &mut b.actions[action.code() as usize];
            s.pressed = true;
            s.held = true;
        });
    }
}

pub fn action_up(code: u8) {
    if let Some(action) = InputAction::from_code(code) {
        with_bridge_mut(|b| {
            b.actions[action.code() as usize].held = false;
        });
    }
}

pub fn take_virtual_action(action: InputAction) -> VirtualActionState {
    with_bridge_mut(|b| {
        let idx = action.code() as usize;
        let state = b.actions[idx];
        b.actions[idx].pressed = false;
        state
    })
}

pub fn peek_virtual_action(action: InputAction) -> VirtualActionState {
    with_bridge(|b| b.actions[action.code() as usize])
}

pub fn clear_virtual_pressed(action: InputAction) {
    with_bridge_mut(|b| b.actions[action.code() as usize].pressed = false);
}

pub fn push_ui_action(action: UiAction) {
    with_bridge_mut(|b| b.pending_ui_actions.push_back(action));
}

pub fn drain_ui_actions() -> Vec<UiAction> {
    with_bridge_mut(|b| b.pending_ui_actions.drain(..).collect())
}

pub fn set_ui_state_json(json: String) {
    with_bridge_mut(|b| b.ui_state_json = json);
}

pub fn ui_state_json() -> String {
    with_bridge(|b| b.ui_state_json.clone())
}

pub fn set_mobile_controls_mode(mode: u8) {
    with_bridge_mut(|b| b.mobile_controls_mode = mode.min(2));
}

pub fn mobile_controls_mode() -> u8 {
    with_bridge(|b| b.mobile_controls_mode)
}

// --- Wasm exports for JS ---

#[no_mangle]
pub extern "C" fn enzo_action_down(code: u8) {
    action_down(code);
}

#[no_mangle]
pub extern "C" fn enzo_action_up(code: u8) {
    action_up(code);
}

#[no_mangle]
pub extern "C" fn enzo_loadout_assign(slot: u8, skill: u8) {
    push_ui_action(UiAction::LoadoutAssign { slot, skill });
}

#[no_mangle]
pub extern "C" fn enzo_loadout_clear(slot: u8) {
    push_ui_action(UiAction::LoadoutClear { slot });
}

#[no_mangle]
pub extern "C" fn enzo_equip_item(item_code: u8) {
    push_ui_action(UiAction::EquipInventoryItem { item: item_code });
}

#[no_mangle]
pub extern "C" fn enzo_unequip_slot(slot_code: u8) {
    push_ui_action(UiAction::UnequipSlot { slot: slot_code });
}

#[no_mangle]
pub extern "C" fn enzo_set_character(character_code: u8) {
    push_ui_action(UiAction::SetPlayableCharacter {
        character: character_code,
    });
}

#[no_mangle]
pub extern "C" fn enzo_set_visual_profile_field(field: u8, value: u8) {
    push_ui_action(UiAction::SetVisualProfileField { field, value });
}

/// Generic UI action bridge.
/// Codes:
/// 1=menu back, 2=open saves, 3=open options, 4=start(slot=a,new=b),
/// 5=resume, 6=request delete(slot=a), 7=cancel delete, 8=confirm delete,
/// 9=set language(a), 10=set mobile controls(a), 11=set interface scale percent(a*256+b),
/// 12=allocate attribute(a), 13=confirm level up, 14=restart, 15=open menu, 16=open skills,
/// 19=equip inventory item(a=item code), 20=unequip slot(a=slot code),
/// 21=set playable character(a=code), 22=set visual profile field(a=field,b=value)
#[no_mangle]
pub extern "C" fn enzo_ui_action(code: u8, a: u8, b: u8, c: u8) {
    if let Some(action) = UiAction::from_ffi(code, a, b, c) {
        push_ui_action(action);
    }
}

thread_local! {
    static UI_STATE_BUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn refresh_ui_state_buf() -> usize {
    UI_STATE_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();
        buf.extend_from_slice(ui_state_json().as_bytes());
        buf.len()
    })
}

#[no_mangle]
pub extern "C" fn enzo_ui_state_ptr() -> *const u8 {
    refresh_ui_state_buf();
    UI_STATE_BUF.with(|buf| buf.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn enzo_ui_state_len() -> u32 {
    UI_STATE_BUF.with(|buf| buf.borrow().len() as u32)
}

#[no_mangle]
pub extern "C" fn enzo_mobile_controls_mode() -> u8 {
    mobile_controls_mode()
}

#[cfg(test)]
mod tests {
    use super::*;
    use enzo_core::input::InputAction;

    #[test]
    fn virtual_action_roundtrip() {
        action_up(InputAction::Slot1.code());
        let _ = take_virtual_action(InputAction::Slot1);

        action_down(InputAction::Slot1.code());
        let s = peek_virtual_action(InputAction::Slot1);
        assert!(s.pressed && s.held);
        let t = take_virtual_action(InputAction::Slot1);
        assert!(t.pressed && t.held);
        let after = peek_virtual_action(InputAction::Slot1);
        assert!(!after.pressed && after.held);
        action_up(InputAction::Slot1.code());
        assert!(!peek_virtual_action(InputAction::Slot1).held);
    }

    #[test]
    fn ignores_unknown_action_codes() {
        action_down(255);
        action_up(255);
        enzo_ui_action(255, 0, 0, 0);
    }

    #[test]
    fn ui_action_queue_drains() {
        let _ = drain_ui_actions();
        push_ui_action(UiAction::MenuBack);
        push_ui_action(UiAction::HudOpenMenu);
        let actions = drain_ui_actions();
        assert_eq!(actions.len(), 2);
        assert!(drain_ui_actions().is_empty());
    }

    #[test]
    fn parses_ffi_ui_actions() {
        assert_eq!(UiAction::from_ffi(1, 0, 0, 0), Some(UiAction::MenuBack));
        assert_eq!(
            UiAction::from_ffi(4, 2, 1, 0),
            Some(UiAction::MenuStart {
                slot: 2,
                new_game: true
            })
        );
        assert_eq!(UiAction::from_ffi(255, 0, 0, 0), None);
    }
}
