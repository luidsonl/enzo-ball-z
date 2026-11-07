//! Keyboard-to-[`PlayerCommand`] translation.
//!
//! Physical key bindings are defined in [`defaults`] and stored in
//! [`KeyBindings`]. This module polls the keyboard (via macroquad), resolves
//! bound [`InputAction`]s, and produces core commands. It does not implement
//! game rules.

pub mod bindings;
pub mod defaults;

use enzo_core::input::{
    axis, ActionSlot, Attribute, InputAction, PlayerCommand, SixSevenKey, SlotInput,
};
use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};

pub use bindings::{EmulatedControlView, KeyBindings};

#[derive(Clone, Debug)]
pub struct InputState {
    pub bindings: KeyBindings,

    left: bool,
    right: bool,
    up: bool,
    down: bool,
    slots_pressed: [bool; ActionSlot::COUNT],
    slots_held: [bool; ActionSlot::COUNT],
    six: bool,
    seven: bool,
    allocate: Option<Attribute>,
    confirm: bool,
    restart: bool,
    menu: bool,
    open_skills: bool,
}

impl Default for InputState {
    fn default() -> Self {
        Self::with_bindings(KeyBindings::default())
    }
}

impl InputState {
    pub fn with_bindings(bindings: KeyBindings) -> Self {
        Self {
            bindings,
            left: false,
            right: false,
            up: false,
            down: false,
            slots_pressed: [false; ActionSlot::COUNT],
            slots_held: [false; ActionSlot::COUNT],
            six: false,
            seven: false,
            allocate: None,
            confirm: false,
            restart: false,
            menu: false,
            open_skills: false,
        }
    }

    /// View over current bindings for emulated/virtual controls and HUD.
    pub fn binding_view(&self) -> &dyn EmulatedControlView {
        &self.bindings
    }

    pub fn poll(&mut self) {
        self.left = self.action_down(InputAction::MoveLeft);
        self.right = self.action_down(InputAction::MoveRight);
        self.up = self.action_down(InputAction::MoveUp);
        self.down = self.action_down(InputAction::MoveDown);

        for (index, slot) in ActionSlot::ALL.iter().copied().enumerate() {
            let Some(action) = input_action_for_slot(slot) else {
                continue;
            };
            self.slots_held[index] = self.action_down(action);
            self.slots_pressed[index] |= self.action_pressed(action);
        }

        self.six |= self.action_pressed(InputAction::Six);
        self.seven |= self.action_pressed(InputAction::Seven);
        self.confirm |= self.action_pressed(InputAction::Confirm);
        self.restart |= self.action_pressed(InputAction::Restart);
        self.menu |= self.action_pressed(InputAction::OpenMenu);
        self.open_skills |= self.action_pressed(InputAction::OpenSkills);

        if self.allocate.is_none() {
            self.allocate = InputAction::ALL
                .into_iter()
                .find(|action| action.attribute().is_some() && self.action_pressed(*action))
                .and_then(|action| action.attribute());
        }
    }

    fn action_down(&self, action: InputAction) -> bool {
        any_key(self.bindings.keys(action), is_key_down)
    }

    fn action_pressed(&self, action: InputAction) -> bool {
        any_key(self.bindings.keys(action), is_key_pressed)
    }

    pub fn take_command(&mut self) -> PlayerCommand {
        let move_x = axis(self.left, self.right);
        let move_y = axis(self.up, self.down);
        let mut slots = [SlotInput::default(); ActionSlot::COUNT];
        for index in 0..ActionSlot::COUNT {
            slots[index] = SlotInput {
                pressed: self.slots_pressed[index],
                held: self.slots_held[index],
            };
        }
        let command = PlayerCommand {
            move_x,
            move_y,
            slots,
            sixseven: if self.six {
                Some(SixSevenKey::Six)
            } else if self.seven {
                Some(SixSevenKey::Seven)
            } else {
                None
            },
            allocate: self.allocate,
            confirm: self.confirm,
            restart: self.restart,
        };
        self.slots_pressed = [false; ActionSlot::COUNT];
        self.six = false;
        self.seven = false;
        self.allocate = None;
        self.confirm = false;
        self.restart = false;
        command
    }

    pub fn take_allocate(&mut self) -> Option<Attribute> {
        let attribute = self.allocate;
        self.allocate = None;
        attribute
    }

    pub fn take_menu(&mut self) -> bool {
        let menu = self.menu;
        self.menu = false;
        menu
    }

    pub fn take_confirm(&mut self) -> bool {
        let confirm = self.confirm;
        self.confirm = false;
        confirm
    }

    pub fn take_restart(&mut self) -> bool {
        let restart = self.restart;
        self.restart = false;
        restart
    }

    pub fn take_menu_request(&mut self) -> bool {
        self.take_menu()
    }

    pub fn take_open_skills(&mut self) -> bool {
        let open = self.open_skills;
        self.open_skills = false;
        open
    }

    pub fn has_activity(&self) -> bool {
        self.left
            || self.right
            || self.up
            || self.down
            || self.slots_held.iter().any(|v| *v)
            || self.slots_pressed.iter().any(|v| *v)
            || self.six
            || self.seven
            || self.allocate.is_some()
            || self.confirm
            || self.restart
            || self.menu
            || self.open_skills
    }

    pub fn allocate_pressed(&self, attribute: Attribute) -> bool {
        self.allocate == Some(attribute)
    }
}

fn input_action_for_slot(slot: ActionSlot) -> Option<InputAction> {
    match slot {
        ActionSlot::Slot1 => Some(InputAction::Slot1),
        ActionSlot::Slot2 => Some(InputAction::Slot2),
        ActionSlot::Slot3 => Some(InputAction::Slot3),
        ActionSlot::Slot4 => Some(InputAction::Slot4),
    }
}

fn any_key(keys: &[KeyCode], f: impl Fn(KeyCode) -> bool) -> bool {
    keys.iter().copied().any(f)
}

#[cfg(test)]
mod tests {
    use super::*;
    use enzo_core::input::ActionSlot;

    #[test]
    fn default_input_uses_default_bindings() {
        let input = InputState::default();
        assert!(input
            .bindings
            .keys(InputAction::MoveLeft)
            .contains(&KeyCode::A));
        assert!(input
            .bindings
            .keys(InputAction::MoveLeft)
            .contains(&KeyCode::Left));
        assert!(input
            .bindings
            .keys(InputAction::Slot1)
            .contains(&KeyCode::Space));
        assert!(input
            .bindings
            .keys(InputAction::Slot1)
            .contains(&KeyCode::W));
    }

    #[test]
    fn binding_view_exposes_selected_keys_to_emulated_controls() {
        let mut bindings = KeyBindings::empty();
        bindings.set_keys(InputAction::MoveLeft, vec![KeyCode::A, KeyCode::Left]);
        bindings.set_keys(InputAction::Slot1, vec![KeyCode::Space, KeyCode::W]);
        let input = InputState::with_bindings(bindings);
        let view = input.binding_view();
        assert_eq!(view.primary_key(InputAction::MoveLeft), Some(KeyCode::A));
        assert!(view.is_key_selected(InputAction::MoveLeft, KeyCode::Left));
        assert!(view.is_key_selected(InputAction::Slot1, KeyCode::W));
    }

    #[test]
    fn consume_preserves_only_continuous_state() {
        let mut input = InputState::default();
        input.left = true;
        input.up = true;
        input.slots_pressed = [true, false, true, false];
        input.slots_held = [true, false, true, true];
        input.six = true;
        input.allocate = Some(Attribute::Aura);
        input.confirm = true;
        input.restart = true;

        let command = input.take_command();
        assert_eq!(command.move_x, -1);
        assert_eq!(command.move_y, -1);
        assert!(command.slots[ActionSlot::Slot1.index()].pressed);
        assert!(command.slots[ActionSlot::Slot1.index()].held);
        assert!(command.sixseven.is_some());
        assert!(command.confirm);
        assert!(command.restart);
        assert_eq!(command.allocate, Some(Attribute::Aura));

        let again = input.take_command();
        assert_eq!(again.move_x, -1);
        assert_eq!(again.move_y, -1);
        assert!(!again.slots[ActionSlot::Slot1.index()].pressed);
        assert!(again.slots[ActionSlot::Slot1.index()].held);
        assert!(again.sixseven.is_none());
        assert!(!again.confirm);
    }

    #[test]
    fn custom_bindings_support_two_keys_for_same_command() {
        let mut bindings = KeyBindings::empty();
        bindings.set_keys(InputAction::MoveLeft, vec![KeyCode::A, KeyCode::Left]);
        bindings.set_keys(InputAction::MoveRight, vec![KeyCode::D, KeyCode::Right]);
        bindings.set_keys(
            InputAction::Slot1,
            vec![KeyCode::Space, KeyCode::W, KeyCode::Up],
        );
        let input = InputState::with_bindings(bindings);
        assert_eq!(input.bindings.keys(InputAction::MoveLeft).len(), 2);
        assert_eq!(input.bindings.keys(InputAction::MoveRight).len(), 2);
        assert!(input
            .bindings
            .keys(InputAction::Slot1)
            .contains(&KeyCode::W));
        assert!(input
            .bindings
            .keys(InputAction::Slot1)
            .contains(&KeyCode::Up));
    }
}
