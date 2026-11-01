//! Componente de controle: quem decide a intenção de uma entidade.
//!
//! O core nunca lê teclado. [`Controller::Player`] guarda a última intenção
//! recebida do cliente; [`Controller::Ai`] guarda o comportamento e o estado da
//! IA. Ambos produzem intenção no mesmo contrato (`movement.walk.intent_x` e
//! edges), então os sistemas de movimento/ataque não sabem quem controla.
//!
//! Entidades sem controle (projéteis e itens) usam [`Controller::None`].

use crate::ai::Ai;
use crate::input::PlayerCommand;

/// Última intenção contínua e edges de um jogador.
///
/// `move_x` e `jump_held` são estados; os demais campos são edges consumidos no
/// tick em que a tecla foi pressionada.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerInput {
    pub move_x: i8,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub dash_pressed: bool,
    pub melee_pressed: bool,
    pub mogar_pressed: bool,
}

impl PlayerInput {
    pub fn from_command(command: PlayerCommand) -> Self {
        Self {
            move_x: command.move_x.clamp(-1, 1),
            jump_pressed: command.jump,
            jump_held: command.jump_held,
            dash_pressed: command.dash,
            melee_pressed: command.melee,
            mogar_pressed: command.mogar,
        }
    }

    /// Descarta os edges. Usado quando um frame produz mais de um tick.
    pub fn clear_edges(&mut self) {
        self.jump_pressed = false;
        self.dash_pressed = false;
        self.melee_pressed = false;
        self.mogar_pressed = false;
    }
}

/// Controlador de uma entidade jogável.
///
/// O controlador guarda o input traduzido; a execução continua nos componentes
/// de movimento e ataque. `PlayerData` fica só com progressão e recurso de aura.
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

/// Quem decide a intenção da entidade.
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

    pub fn is_playable(&self) -> bool {
        matches!(self, Self::Player(_))
    }

    pub fn is_ai(&self) -> bool {
        matches!(self, Self::Ai(_))
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
    use crate::input::Attribute;

    #[test]
    fn player_controller_translates_command() {
        let command = PlayerCommand {
            move_x: 2,
            jump: true,
            jump_held: true,
            dash: true,
            melee: true,
            mogar: true,
            sixseven: None,
            allocate: Some(Attribute::Speed),
            confirm: true,
            restart: false,
        };
        let mut controller = Controller::player();
        controller.player_controller_mut().unwrap().apply(command);

        let input = controller.player_controller().unwrap().input;
        assert_eq!(input.move_x, 1);
        assert!(input.jump_pressed);
        assert!(input.jump_held);
        assert!(input.dash_pressed);
        assert!(input.melee_pressed);
        assert!(input.mogar_pressed);
    }

    #[test]
    fn clearing_edges_keeps_continuous_state() {
        let mut controller = PlayerController::default();
        controller.apply(PlayerCommand {
            move_x: -1,
            jump: true,
            jump_held: true,
            ..PlayerCommand::none()
        });

        controller.clear_edges();

        assert_eq!(controller.input.move_x, -1);
        assert!(controller.input.jump_held);
        assert!(!controller.input.jump_pressed);
    }

    #[test]
    fn controller_kind_is_explicit() {
        assert!(Controller::player().is_playable());
        assert!(Controller::from_ai(Ai::slime()).is_ai());
        assert!(!Controller::None.is_playable());
        assert!(Controller::None.ai().is_none());
    }
}
