//! Keyboard-to-[`PlayerCommand`] translation.

use enzo_core::input::{ActionSlot, Attribute, PlayerCommand, SixSevenKey, SlotInput};
use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub up: bool,
    pub down: bool,
    slots_pressed: [bool; ActionSlot::COUNT],
    slots_held: [bool; ActionSlot::COUNT],
    six: bool,
    seven: bool,
    allocate: Option<Attribute>,
    confirm: bool,
    restart: bool,
    menu: bool,
}

impl InputState {
    pub fn poll(&mut self) {
        self.left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
        self.right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);
        self.up = is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
        self.down = is_key_down(KeyCode::S) || is_key_down(KeyCode::Down);

        // Up also triggers jump (slot 1) on press.
        let up_pressed = is_key_pressed(KeyCode::W) || is_key_pressed(KeyCode::Up);

        let slot_keys = [
            [KeyCode::Space].as_slice(),
            [KeyCode::LeftShift, KeyCode::RightShift].as_slice(),
            [KeyCode::J].as_slice(),
            [KeyCode::K].as_slice(),
        ];
        for (index, keys) in slot_keys.iter().enumerate() {
            let held = keys.iter().any(|key| is_key_down(*key));
            let pressed = keys.iter().any(|key| is_key_pressed(*key));
            self.slots_held[index] = held;
            self.slots_pressed[index] |= pressed;
        }
        if up_pressed {
            self.slots_pressed[ActionSlot::Slot1.index()] = true;
        }

        self.six |= is_key_pressed(KeyCode::Key6) || is_key_pressed(KeyCode::Kp6);
        self.seven |= is_key_pressed(KeyCode::Key7) || is_key_pressed(KeyCode::Kp7);
        self.confirm |= is_key_pressed(KeyCode::Enter) || is_key_pressed(KeyCode::KpEnter);
        self.restart |= is_key_pressed(KeyCode::R);
        self.menu |= is_key_pressed(KeyCode::Escape);

        self.allocate = if is_key_pressed(KeyCode::Key1) || is_key_pressed(KeyCode::Kp1) {
            Some(Attribute::Speed)
        } else if is_key_pressed(KeyCode::Key2) || is_key_pressed(KeyCode::Kp2) {
            Some(Attribute::Strength)
        } else if is_key_pressed(KeyCode::Key3) || is_key_pressed(KeyCode::Kp3) {
            Some(Attribute::Health)
        } else if is_key_pressed(KeyCode::Key4) || is_key_pressed(KeyCode::Kp4) {
            Some(Attribute::Aura)
        } else {
            self.allocate
        };
    }

    pub fn take_command(&mut self) -> PlayerCommand {
        let move_x = match (self.left, self.right) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        };
        let move_y = match (self.up, self.down) {
            (true, false) => -1,
            (false, true) => 1,
            _ => 0,
        };
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
    }

    pub fn allocate_pressed(&self, attribute: Attribute) -> bool {
        self.allocate == Some(attribute)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn consume_preserves_only_continuous_state() {
        let mut input = InputState {
            left: true,
            up: true,
            slots_pressed: [true, false, true, false],
            slots_held: [true, false, true, true],
            six: true,
            allocate: Some(Attribute::Aura),
            confirm: true,
            restart: true,
            ..InputState::default()
        };

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
    fn horizontal_axis_cancels_opposite_directions() {
        let input = InputState {
            left: true,
            right: true,
            ..InputState::default()
        };
        assert_eq!(
            input,
            InputState {
                left: true,
                right: true,
                ..InputState::default()
            }
        );
    }
}
