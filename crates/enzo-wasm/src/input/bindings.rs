//! Key binding model for semantic input actions.
//!
//! Bindings live in the client because physical keys are platform/input-backend
//! details. The core only consumes [`PlayerCommand`]/semantic intent.
//!
//! Multiple keys can trigger the same action. A key may also be present in more
//! than one action when intentionally configured (for example, Up providing both
//! vertical intent and a jump trigger).

use enzo_core::input::InputAction;
use macroquad::prelude::KeyCode;

/// View over a selected binding, used by emulated/virtual controls and UI.
pub trait EmulatedControlView {
    /// All keys currently bound to `action`, in priority/display order.
    fn bound_keys(&self, action: InputAction) -> &[KeyCode];
    /// Primary key to display for `action`, if any.
    fn primary_key(&self, action: InputAction) -> Option<KeyCode> {
        self.bound_keys(action).first().copied()
    }
    /// Whether `key` is currently selected/bound for `action`.
    fn is_key_selected(&self, action: InputAction, key: KeyCode) -> bool {
        self.bound_keys(action).contains(&key)
    }
}

/// Physical key bindings for [`InputAction`]s.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyBindings {
    keys: Vec<Vec<KeyCode>>,
}

impl Default for KeyBindings {
    fn default() -> Self {
        super::defaults::default_bindings()
    }
}

impl KeyBindings {
    /// Creates an empty binding table with entries for all actions.
    pub fn empty() -> Self {
        Self {
            keys: vec![Vec::new(); InputAction::COUNT],
        }
    }

    /// Returns all keys bound to `action`.
    pub fn keys(&self, action: InputAction) -> &[KeyCode] {
        self.keys
            .get(action.code() as usize)
            .map(Vec::as_slice)
            .unwrap_or(&[])
    }

    /// Returns a mutable key list for `action`.
    pub fn keys_mut(&mut self, action: InputAction) -> &mut Vec<KeyCode> {
        let idx = action.code() as usize;
        if self.keys.len() <= idx {
            self.keys.resize(idx + 1, Vec::new());
        }
        &mut self.keys[idx]
    }

    /// Replaces all keys bound to `action`.
    pub fn set_keys(&mut self, action: InputAction, keys: Vec<KeyCode>) {
        *self.keys_mut(action) = keys;
    }

    /// Adds `key` to `action` if it is not already present.
    pub fn add_key(&mut self, action: InputAction, key: KeyCode) {
        let keys = self.keys_mut(action);
        if !keys.contains(&key) {
            keys.push(key);
        }
    }

    /// Removes all occurrences of `key` from `action`.
    pub fn remove_key(&mut self, action: InputAction, key: KeyCode) {
        self.keys_mut(action).retain(|k| *k != key);
    }

    /// Returns all actions that include `key`.
    pub fn actions_for_key(&self, key: KeyCode) -> Vec<InputAction> {
        InputAction::ALL
            .into_iter()
            .filter(|action| self.keys(*action).contains(&key))
            .collect()
    }

    /// Returns true if `key` is bound to `action`.
    pub fn contains(&self, action: InputAction, key: KeyCode) -> bool {
        self.keys(action).contains(&key)
    }

    /// Returns actions that share at least one key.
    pub fn conflicts(&self) -> Vec<(InputAction, InputAction, KeyCode)> {
        let mut conflicts = Vec::new();
        for (i, a) in InputAction::ALL.iter().copied().enumerate() {
            for b in InputAction::ALL.iter().copied().skip(i + 1) {
                for key in self.keys(a) {
                    if self.keys(b).contains(key) {
                        conflicts.push((a, b, *key));
                    }
                }
            }
        }
        conflicts
    }

    /// Stable display label for a key (for HUD/emulated controls).
    pub fn key_label(key: KeyCode) -> String {
        format!("{key:?}")
    }

    /// Primary key label for `action`, or an em dash when unbound.
    pub fn primary_key_label(&self, action: InputAction) -> String {
        self.primary_key(action)
            .map(Self::key_label)
            .unwrap_or_else(|| "—".to_owned())
    }
}

impl EmulatedControlView for KeyBindings {
    fn bound_keys(&self, action: InputAction) -> &[KeyCode] {
        self.keys(action)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supports_multiple_keys_for_same_action() {
        let mut bindings = KeyBindings::empty();
        bindings.set_keys(InputAction::MoveLeft, vec![KeyCode::A, KeyCode::Left]);
        assert_eq!(bindings.keys(InputAction::MoveLeft).len(), 2);
        assert!(bindings.contains(InputAction::MoveLeft, KeyCode::A));
        assert!(bindings.contains(InputAction::MoveLeft, KeyCode::Left));
    }

    #[test]
    fn supports_same_key_in_multiple_actions() {
        let mut bindings = KeyBindings::empty();
        bindings.set_keys(InputAction::MoveUp, vec![KeyCode::W, KeyCode::Up]);
        bindings.set_keys(InputAction::Slot1, vec![KeyCode::Space, KeyCode::W]);
        assert!(bindings.contains(InputAction::MoveUp, KeyCode::W));
        assert!(bindings.contains(InputAction::Slot1, KeyCode::W));
        let actions = bindings.actions_for_key(KeyCode::W);
        assert!(actions.contains(&InputAction::MoveUp));
        assert!(actions.contains(&InputAction::Slot1));
    }

    #[test]
    fn emulated_control_view_exposes_selected_key() {
        let mut bindings = KeyBindings::empty();
        bindings.set_keys(InputAction::Slot1, vec![KeyCode::Space, KeyCode::W]);
        let view: &dyn EmulatedControlView = &bindings;
        assert_eq!(view.primary_key(InputAction::Slot1), Some(KeyCode::Space));
        assert!(view.is_key_selected(InputAction::Slot1, KeyCode::W));
        assert_eq!(view.bound_keys(InputAction::Slot1).len(), 2);
    }

    #[test]
    fn detects_conflicts() {
        let mut bindings = KeyBindings::empty();
        bindings.set_keys(InputAction::MoveUp, vec![KeyCode::W]);
        bindings.set_keys(InputAction::Slot1, vec![KeyCode::W, KeyCode::Space]);
        let conflicts = bindings.conflicts();
        assert!(conflicts.iter().any(|(a, b, k)| {
            *k == KeyCode::W
                && ((*a == InputAction::MoveUp && *b == InputAction::Slot1)
                    || (*a == InputAction::Slot1 && *b == InputAction::MoveUp))
        }));
    }

    #[test]
    fn add_and_remove_key() {
        let mut bindings = KeyBindings::empty();
        bindings.add_key(InputAction::MoveLeft, KeyCode::A);
        bindings.add_key(InputAction::MoveLeft, KeyCode::A);
        bindings.add_key(InputAction::MoveLeft, KeyCode::Left);
        assert_eq!(bindings.keys(InputAction::MoveLeft).len(), 2);
        bindings.remove_key(InputAction::MoveLeft, KeyCode::A);
        assert!(!bindings.contains(InputAction::MoveLeft, KeyCode::A));
        assert!(bindings.contains(InputAction::MoveLeft, KeyCode::Left));
    }
}
