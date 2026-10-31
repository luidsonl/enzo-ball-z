//! Contrato de input. O core recebe comandos, nunca o teclado.

use crate::MAX_PLAYERS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FarmKey {
    Six,
    Seven,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Attribute {
    Speed,
    Strength,
    Health,
    Aura,
}

impl Attribute {
    pub const ALL: [Attribute; 4] = [
        Attribute::Speed,
        Attribute::Strength,
        Attribute::Health,
        Attribute::Aura,
    ];
}

/// Intenção de um jogador em um único tick.
///
/// `jump`, `melee`, `mogar`, `farm`, `allocate`, `confirm` e `restart` são
/// edge-triggered: o cliente envia `true` apenas no tick em que a tecla foi
/// pressionada. `jump_held` e `move_x` são estados contínuos.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerCommand {
    pub move_x: i8,
    pub jump: bool,
    pub jump_held: bool,
    pub melee: bool,
    pub mogar: bool,
    pub farm: Option<FarmKey>,
    pub allocate: Option<Attribute>,
    pub confirm: bool,
    pub restart: bool,
}

impl PlayerCommand {
    pub fn none() -> Self {
        Self::default()
    }

    /// Mantém apenas os estados contínuos. Usado quando um frame produz mais de
    /// um tick: o edge não pode ser aplicado duas vezes.
    pub fn without_edges(mut self) -> Self {
        self.jump = false;
        self.melee = false;
        self.mogar = false;
        self.farm = None;
        self.allocate = None;
        self.confirm = false;
        self.restart = false;
        self
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct TickInput {
    pub commands: [Option<PlayerCommand>; MAX_PLAYERS],
}

impl TickInput {
    pub fn single(command: PlayerCommand) -> Self {
        let mut input = Self::default();
        input.commands[0] = Some(command);
        input
    }

    pub fn with_player(mut self, player: usize, command: PlayerCommand) -> Self {
        if player < MAX_PLAYERS {
            self.commands[player] = Some(command);
        }
        self
    }

    pub fn command(&self, player: usize) -> Option<PlayerCommand> {
        self.commands.get(player).copied().flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_sets_first_player() {
        let input = TickInput::single(PlayerCommand {
            move_x: 1,
            jump: true,
            ..PlayerCommand::none()
        });
        let command = input.command(0).unwrap();
        assert_eq!(command.move_x, 1);
        assert!(command.jump);
        assert!(input.command(1).is_none());
    }

    #[test]
    fn with_player_ignores_out_of_range() {
        let input = TickInput::default().with_player(MAX_PLAYERS, PlayerCommand::none());
        assert!(input.commands.iter().all(Option::is_none));
    }

    #[test]
    fn without_edges_keeps_held_state() {
        let command = PlayerCommand {
            move_x: -1,
            jump: true,
            jump_held: true,
            melee: true,
            mogar: true,
            farm: Some(FarmKey::Six),
            allocate: Some(Attribute::Aura),
            confirm: true,
            restart: true,
        }
        .without_edges();

        assert_eq!(command.move_x, -1);
        assert!(command.jump_held);
        assert!(!command.jump);
        assert!(!command.melee);
        assert!(!command.mogar);
        assert!(command.farm.is_none());
        assert!(command.allocate.is_none());
        assert!(!command.confirm);
        assert!(!command.restart);
    }
}
