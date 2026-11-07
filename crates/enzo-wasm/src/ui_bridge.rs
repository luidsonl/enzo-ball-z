//! DOM/HTML UI bridge.
//!
//! The HTML/mobile UI emits semantic [`InputAction`] codes and loadout requests.
//! Rust exports functions for JS to call, and merges virtual actions into input.
//! This bridge must not contain game rules.

use std::cell::RefCell;
use std::collections::VecDeque;

use enzo_core::input::InputAction;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct VirtualActionState {
    pub pressed: bool,
    pub held: bool,
}

#[derive(Clone, Debug, Default)]
pub struct BridgeState {
    pub actions: [VirtualActionState; InputAction::COUNT],
    pub ui_state_json: String,
    pub ui_state_dirty: bool,
    pub pending_loadout: VecDeque<(u8, u8)>, // (slot, skill) - placeholder protocol
    pub pending_clear: VecDeque<u8>,
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
            let s = &mut b.actions[action.code() as usize];
            s.held = false;
        });
    }
}

/// Takes accumulated virtual state for `action` and clears its pressed edge.
pub fn take_virtual_action(action: InputAction) -> VirtualActionState {
    with_bridge_mut(|b| {
        let idx = action.code() as usize;
        let state = b.actions[idx];
        b.actions[idx].pressed = false;
        // Keep held until action_up is received.
        state
    })
}

/// Peeks virtual held/pressed without consuming pressed (used for polling merge).
pub fn peek_virtual_action(action: InputAction) -> VirtualActionState {
    with_bridge(|b| b.actions[action.code() as usize])
}

pub fn clear_virtual_pressed(action: InputAction) {
    with_bridge_mut(|b| b.actions[action.code() as usize].pressed = false);
}

pub fn set_ui_state_json(json: String) {
    with_bridge_mut(|b| {
        b.ui_state_dirty = b.ui_state_json != json;
        b.ui_state_json = json;
    });
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

// --- Wasm exports for JS (miniquad/macroquad style) ---

/// JS -> Rust: semantic action pressed.
#[no_mangle]
pub extern "C" fn enzo_action_down(code: u8) {
    action_down(code);
}

/// JS -> Rust: semantic action released.
#[no_mangle]
pub extern "C" fn enzo_action_up(code: u8) {
    action_up(code);
}

/// JS -> Rust: request loadout assignment (slot index, skill code). Placeholder.
#[no_mangle]
pub extern "C" fn enzo_loadout_assign(slot: u8, skill: u8) {
    with_bridge_mut(|b| b.pending_loadout.push_back((slot, skill)));
}

/// JS -> Rust: request loadout clear (slot index). Placeholder.
#[no_mangle]
pub extern "C" fn enzo_loadout_clear(slot: u8) {
    with_bridge_mut(|b| b.pending_clear.push_back(slot));
}

thread_local! {
    static UI_STATE_BUF: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
}

fn refresh_ui_state_buf() -> usize {
    UI_STATE_BUF.with(|buf| {
        let mut buf = buf.borrow_mut();
        let json = ui_state_json();
        buf.clear();
        buf.extend_from_slice(json.as_bytes());
        buf.len()
    })
}

/// Returns pointer to UTF-8 UI state JSON bytes (valid until next refresh call).
#[no_mangle]
pub extern "C" fn enzo_ui_state_ptr() -> *const u8 {
    refresh_ui_state_buf();
    UI_STATE_BUF.with(|buf| buf.borrow().as_ptr())
}

/// Returns length of the last refreshed UI state JSON buffer.
#[no_mangle]
pub extern "C" fn enzo_ui_state_len() -> u32 {
    UI_STATE_BUF.with(|buf| buf.borrow().len() as u32)
}

/// Returns mobile controls mode: 0=off, 1=on, 2=auto.
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
        // Reset relevant action
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
    }
}
