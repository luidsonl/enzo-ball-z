//! Default keyboard bindings.
//!
//! Defaults are defined in one place so gameplay/UI/input stay consistent.
//! Multiple physical keys may trigger the same action (e.g. WASD + arrows).
//! A single key may also contribute to more than one action when intentionally
//! configured (e.g. Up triggers both vertical movement intent and jump/slot 1).

use enzo_core::input::InputAction;
use macroquad::prelude::KeyCode;

use super::bindings::KeyBindings;

/// Returns the default key bindings for the local player.
pub fn default_bindings() -> KeyBindings {
    let mut bindings = KeyBindings::empty();
    bind(
        &mut bindings,
        InputAction::MoveLeft,
        &[KeyCode::A, KeyCode::Left],
    );
    bind(
        &mut bindings,
        InputAction::MoveRight,
        &[KeyCode::D, KeyCode::Right],
    );
    bind(
        &mut bindings,
        InputAction::MoveUp,
        &[KeyCode::W, KeyCode::Up],
    );
    bind(
        &mut bindings,
        InputAction::MoveDown,
        &[KeyCode::S, KeyCode::Down],
    );

    // Slot 1 is Jump by default. A key cannot perform more than one action,
    // so jump uses Space by default (movement/climb/fly use W/Up).
    bind(&mut bindings, InputAction::Slot1, &[KeyCode::Space]);
    bind(
        &mut bindings,
        InputAction::Slot2,
        &[KeyCode::LeftShift, KeyCode::RightShift],
    );
    bind(&mut bindings, InputAction::Slot3, &[KeyCode::J]);
    bind(&mut bindings, InputAction::Slot4, &[KeyCode::K]);

    bind(
        &mut bindings,
        InputAction::Six,
        &[KeyCode::Key6, KeyCode::Kp6],
    );
    bind(
        &mut bindings,
        InputAction::Seven,
        &[KeyCode::Key7, KeyCode::Kp7],
    );

    // Attribute allocation is done via the game UI (not keyboard shortcuts).
    bind(&mut bindings, InputAction::AllocateSpeed, &[]);
    bind(&mut bindings, InputAction::AllocateStrength, &[]);
    bind(&mut bindings, InputAction::AllocateHealth, &[]);
    bind(&mut bindings, InputAction::AllocateAura, &[]);
    bind(&mut bindings, InputAction::AllocateEgo, &[]);

    bind(
        &mut bindings,
        InputAction::Confirm,
        &[KeyCode::Enter, KeyCode::KpEnter],
    );
    bind(&mut bindings, InputAction::Restart, &[KeyCode::R]);
    bind(&mut bindings, InputAction::OpenMenu, &[KeyCode::Escape]);
    // OpenSkills is intentionally left unbound by default until the UI flow
    // and browser-focus handling are finalized.
    bind(&mut bindings, InputAction::OpenSkills, &[]);

    bindings
}

fn bind(bindings: &mut KeyBindings, action: InputAction, keys: &[KeyCode]) {
    bindings.set_keys(action, keys.to_vec());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_define_multiple_keys_for_movement() {
        let bindings = default_bindings();
        assert!(bindings.keys(InputAction::MoveLeft).contains(&KeyCode::A));
        assert!(bindings
            .keys(InputAction::MoveLeft)
            .contains(&KeyCode::Left));
        assert!(bindings.keys(InputAction::MoveRight).contains(&KeyCode::D));
        assert!(bindings
            .keys(InputAction::MoveRight)
            .contains(&KeyCode::Right));
    }

    #[test]
    fn defaults_keep_jump_and_move_keys_exclusive() {
        let bindings = default_bindings();
        assert!(bindings.keys(InputAction::MoveUp).contains(&KeyCode::Up));
        assert!(bindings.keys(InputAction::MoveUp).contains(&KeyCode::W));
        assert!(bindings.keys(InputAction::Slot1).contains(&KeyCode::Space));
        assert!(!bindings.keys(InputAction::Slot1).contains(&KeyCode::Up));
        assert!(!bindings.keys(InputAction::Slot1).contains(&KeyCode::W));
        assert!(bindings.conflicts().is_empty());
    }

    #[test]
    fn defaults_cover_all_actions() {
        let bindings = default_bindings();
        for action in InputAction::ALL {
            // Every action has an entry (possibly empty for OpenSkills).
            let _ = bindings.keys(action);
        }
        assert!(!bindings.keys(InputAction::Slot1).is_empty());
        assert!(bindings.keys(InputAction::OpenSkills).is_empty());
    }
}
