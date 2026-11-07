//! Componente de controle: quem decide a intenção de uma entidade.
//!
//! O core nunca lê teclado. [`Controller::Player`] guarda a última intenção
//! recebida do cliente; [`Controller::Ai`] guarda o comportamento e o estado da
//! IA. Ambos produzem intenção no mesmo contrato (`locomotion.walk.intent_x` e
//! slots), então os sistemas não sabem quem controla.
//!
//! Entidades sem controle (projéteis e itens) usam [`Controller::None`].

use crate::ai::Ai;
use crate::input::{ActionSlot, PlayerCommand, SlotInput};

/// Última intenção contínua e edges de um jogador.
///
/// `move_x`/`move_y` e `slots[*].held` são estados; `slots[*].pressed` é edge.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerInput {
    pub move_x: i8,
    pub move_y: i8,
    pub slots: [SlotInput; ActionSlot::COUNT],
}

impl PlayerInput {
    pub fn from_command(command: PlayerCommand) -> Self {
        Self {
            move_x: command.move_x.clamp(-1, 1),
            move_y: command.move_y.clamp(-1, 1),
            slots: command.slots,
        }
    }

    /// Zera apenas os edges. O estado contínuo permanece até o próximo frame.
    pub fn clear_edges(&mut self) {
        for slot in &mut self.slots {
            slot.pressed = false;
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerController {
    pub input: PlayerInput,
}

impl PlayerController {
    pub fn apply(&mut self, command: PlayerCommand) {
        self.input = PlayerInput::from_command(command);
    }

    pub fn clear_edges(&mut self) {
        self.input.clear_edges();
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Controller {
    Player(PlayerController),
    Ai(Ai),
    None,
}

impl Controller {
    pub fn player() -> Self {
        Self::Player(PlayerController::default())
    }

    pub fn from_ai(ai: Ai) -> Self {
        Self::Ai(ai)
    }

    pub fn is_player(&self) -> bool {
        matches!(self, Self::Player(_))
    }

    pub fn is_playable(&self) -> bool {
        matches!(self, Self::Player(_))
    }

    pub fn is_ai(&self) -> bool {
        matches!(self, Self::Ai(_))
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Self::None)
    }

    pub fn player_controller(&self) -> Option<&PlayerController> {
        match self {
            Self::Player(controller) => Some(controller),
            _ => None,
        }
    }

    pub fn player_controller_mut(&mut self) -> Option<&mut PlayerController> {
        match self {
            Self::Player(controller) => Some(controller),
            _ => None,
        }
    }

    pub fn player_input(&self) -> Option<PlayerInput> {
        self.player_controller().map(|controller| controller.input)
    }

    pub fn ai(&self) -> Option<&Ai> {
        match self {
            Self::Ai(ai) => Some(ai),
            _ => None,
        }
    }

    pub fn ai_mut(&mut self) -> Option<&mut Ai> {
        match self {
            Self::Ai(ai) => Some(ai),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::{ActionSlot, PlayerCommand, SlotInput};

    #[test]
    fn player_controller_translates_command() {
        let mut slots = [SlotInput::default(); ActionSlot::COUNT];
        slots[ActionSlot::Slot1.index()] = SlotInput::pressed();
        let command = PlayerCommand {
            move_x: 2,
            move_y: -1,
            slots,
            ..PlayerCommand::none()
        };
        let mut controller = Controller::player();
        controller.player_controller_mut().unwrap().apply(command);

        let input = controller.player_controller().unwrap().input;
        assert_eq!(input.move_x, 1);
        assert_eq!(input.move_y, -1);
        assert!(input.slots[ActionSlot::Slot1.index()].pressed);
    }

    #[test]
    fn clearing_edges_keeps_continuous_state() {
        let mut controller = PlayerController::default();
        let mut slots = [SlotInput::default(); ActionSlot::COUNT];
        slots[0] = SlotInput {
            pressed: true,
            held: true,
        };
        controller.apply(PlayerCommand {
            move_x: -1,
            slots,
            ..PlayerCommand::none()
        });

        controller.clear_edges();

        assert_eq!(controller.input.move_x, -1);
        assert!(controller.input.slots[0].held);
        assert!(!controller.input.slots[0].pressed);
    }

    #[test]
    fn controller_kind_is_explicit() {
        assert!(Controller::player().is_playable());
        assert!(Controller::from_ai(Ai::slime()).is_ai());
        assert!(!Controller::None.is_playable());
        assert!(Controller::None.ai().is_none());
    }
}
