//! Tradução do teclado para [`PlayerCommand`].
//!
//! O core é edge-triggered. O cliente acumula edges entre frames e os consome
//! apenas no primeiro tick do frame; ticks extras usam `without_edges()` no
//! `app`. Isso evita perder uma tecla pressionada em um frame sem tick e evita
//! aplicar o mesmo edge duas vezes quando o frame produz vários ticks.

use enzo_core::input::{Attribute, PlayerCommand, SixSevenKey};
use macroquad::prelude::{is_key_down, is_key_pressed, KeyCode};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct InputState {
    pub left: bool,
    pub right: bool,
    pub jump_held: bool,
    jump: bool,
    dash: bool,
    melee: bool,
    mogar: bool,
    six: bool,
    seven: bool,
    allocate: Option<Attribute>,
    confirm: bool,
    restart: bool,
}

impl InputState {
    /// Lê o teclado uma vez por frame. Edges ficam pendentes até um tick.
    pub fn poll(&mut self) {
        self.left = is_key_down(KeyCode::A) || is_key_down(KeyCode::Left);
        self.right = is_key_down(KeyCode::D) || is_key_down(KeyCode::Right);

        let jump_down =
            is_key_down(KeyCode::Space) || is_key_down(KeyCode::W) || is_key_down(KeyCode::Up);
        self.jump_held = jump_down;

        self.jump |= is_key_pressed(KeyCode::Space)
            || is_key_pressed(KeyCode::W)
            || is_key_pressed(KeyCode::Up);
        self.dash |= is_key_pressed(KeyCode::LeftShift) || is_key_pressed(KeyCode::RightShift);
        self.melee |= is_key_pressed(KeyCode::J);
        self.mogar |= is_key_pressed(KeyCode::K);
        self.six |= is_key_pressed(KeyCode::Key6);
        self.seven |= is_key_pressed(KeyCode::Key7);

        if is_key_pressed(KeyCode::Key1) {
            self.allocate = Some(Attribute::Speed);
        } else if is_key_pressed(KeyCode::Key2) {
            self.allocate = Some(Attribute::Strength);
        } else if is_key_pressed(KeyCode::Key3) {
            self.allocate = Some(Attribute::Health);
        } else if is_key_pressed(KeyCode::Key4) {
            self.allocate = Some(Attribute::Aura);
        }

        self.confirm |= is_key_pressed(KeyCode::Enter);
        self.restart |= is_key_pressed(KeyCode::R);
    }

    /// Monta o comando do primeiro tick do frame e limpa os edges consumidos.
    pub fn take_command(&mut self) -> PlayerCommand {
        let command = PlayerCommand {
            move_x: axis(self.left, self.right),
            jump: self.jump,
            jump_held: self.jump_held,
            dash: self.dash,
            melee: self.melee,
            mogar: self.mogar,
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
        self.clear_edges();
        command
    }

    /// Descarta edges pendentes. Estados contínuos (direção e pulo segurado)
    /// continuam valendo para os próximos ticks.
    pub fn clear_edges(&mut self) {
        self.jump = false;
        self.dash = false;
        self.melee = false;
        self.mogar = false;
        self.six = false;
        self.seven = false;
        self.allocate = None;
        self.confirm = false;
        self.restart = false;
    }

    /// `true` se alguma tecla de jogo está pressionada. Usado para esconder a
    /// dica de controles depois que o jogador começa a agir.
    pub fn has_activity(&self) -> bool {
        self.left
            || self.right
            || self.jump_held
            || self.jump
            || self.dash
            || self.melee
            || self.mogar
            || self.six
            || self.seven
            || self.allocate.is_some()
            || self.confirm
            || self.restart
    }
}

/// Eixo digital com histerese zero: esquerda e direita juntas param o Enzo.
pub fn axis(left: bool, right: bool) -> i8 {
    match (left, right) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn axis_is_digital_and_cancels_opposites() {
        assert_eq!(axis(false, false), 0);
        assert_eq!(axis(true, false), -1);
        assert_eq!(axis(false, true), 1);
        assert_eq!(axis(true, true), 0);
    }

    #[test]
    fn default_command_has_no_edges() {
        let mut input = InputState::default();
        assert_eq!(input.take_command(), PlayerCommand::none());
    }

    #[test]
    fn clearing_edges_keeps_held_direction_and_jump() {
        let mut input = InputState {
            left: true,
            jump_held: true,
            jump: true,
            dash: true,
            ..InputState::default()
        };

        let command = input.take_command();
        assert_eq!(command.move_x, -1);
        assert!(command.jump);
        assert!(command.jump_held);
        assert!(command.dash);

        let next = input.take_command();
        assert_eq!(next.move_x, -1);
        assert!(next.jump_held);
        assert!(!next.jump);
        assert!(!next.dash);
    }

    #[test]
    fn activity_tracks_edges_and_held_state() {
        let mut input = InputState::default();
        assert!(!input.has_activity());
        input.right = true;
        assert!(input.has_activity());
        input.right = false;
        input.confirm = true;
        assert!(input.has_activity());
    }
}
