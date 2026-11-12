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

    /// Removes `key` from every action except `keep_in` (if provided).
    pub fn remove_key_globally(&mut self, key: KeyCode, keep_in: Option<InputAction>) {
        for action in InputAction::ALL {
            if Some(action) == keep_in {
                continue;
            }
            self.remove_key(action, key);
        }
    }

    /// Binds `key` to `action`, ensuring the key is not used by other actions.
    /// If `replace` is true, action keeps only this key; otherwise the key is added/made primary.
    pub fn bind_key_exclusive(&mut self, action: InputAction, key: KeyCode, replace: bool) {
        self.remove_key_globally(key, Some(action));
        if replace {
            self.set_keys(action, vec![key]);
            return;
        }
        let keys = self.keys_mut(action);
        keys.retain(|k| *k != key);
        keys.insert(0, key);
    }

    /// Enforces invariant: a key may belong to at most one action.
    /// Keeps the first occurrence in `InputAction::ALL` order.
    pub fn sanitize_exclusive_keys(&mut self) {
        use std::collections::HashSet;
        let mut seen: HashSet<KeyCode> = HashSet::new();
        for action in InputAction::ALL {
            let mut unique = Vec::new();
            for key in self.keys(action).iter().copied() {
                if seen.insert(key) {
                    unique.push(key);
                }
            }
            self.set_keys(action, unique);
        }
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

    /// Serialize bindings to a JSON-compatible map: action code (string) -> key names.
    pub fn to_serializable(&self) -> serde_json::Map<String, serde_json::Value> {
        let mut map = serde_json::Map::new();
        for action in InputAction::ALL {
            let names: Vec<serde_json::Value> = self
                .keys(action)
                .iter()
                .filter_map(|k| {
                    super::keycode::key_to_name(*k).map(|n| serde_json::Value::String(n.to_owned()))
                })
                .collect();
            map.insert(action.code().to_string(), serde_json::Value::Array(names));
        }
        map
    }

    /// Deserialize bindings from a map produced by [`Self::to_serializable`].
    /// Unknown actions/keys are ignored. Aura farming (Six/Seven) is always kept at defaults.
    pub fn from_serializable(value: &serde_json::Value) -> Option<Self> {
        let obj = value.as_object()?;
        let defaults = Self::default();
        let mut bindings = defaults.clone();
        for (k, v) in obj {
            let Ok(code) = k.parse::<u8>() else { continue };
            let Some(action) = InputAction::from_code(code) else {
                continue;
            };
            if matches!(action, InputAction::Six | InputAction::Seven) {
                continue;
            }
            let Some(arr) = v.as_array() else { continue };
            let keys: Vec<KeyCode> = arr
                .iter()
                .filter_map(|x| x.as_str())
                .filter_map(super::keycode::key_from_name)
                .collect();
            bindings.set_keys(action, keys);
        }
        bindings.set_keys(InputAction::Six, defaults.keys(InputAction::Six).to_vec());
        bindings.set_keys(
            InputAction::Seven,
            defaults.keys(InputAction::Seven).to_vec(),
        );
        // A key cannot perform more than one action.
        bindings.sanitize_exclusive_keys();
        // Re-enforce fixed aura farming keys after sanitization.
        bindings.set_keys(InputAction::Six, defaults.keys(InputAction::Six).to_vec());
        bindings.set_keys(
            InputAction::Seven,
            defaults.keys(InputAction::Seven).to_vec(),
        );
        Some(bindings)
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
