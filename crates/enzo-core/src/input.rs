//! Contrato de input. O core recebe comandos, nunca o teclado.
//!
//! A camada semântica ([`InputAction`]) fica aqui para ser compartilhada entre
//! teclado, touch e a ponte HTML. O cliente traduz `InputAction` para
//! [`PlayerCommand`]; o core só conhece a intenção do tick.
//!
//! [`ActionSlot`] é o botão de habilidade (1–4). Ele não sabe qual skill está
//! equipada: o vínculo vive no [`crate::skills::Loadout`].

use crate::MAX_PLAYERS;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SixSevenKey {
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

/// Slot de habilidade. A ordem é estável e indexa arrays/snapshots.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActionSlot {
    Slot1,
    Slot2,
    Slot3,
    Slot4,
}

impl ActionSlot {
    pub const COUNT: usize = 4;
    pub const ALL: [ActionSlot; Self::COUNT] = [
        ActionSlot::Slot1,
        ActionSlot::Slot2,
        ActionSlot::Slot3,
        ActionSlot::Slot4,
    ];

    /// Índice 0-based usado em arrays e no snapshot.
    pub const fn index(self) -> usize {
        match self {
            ActionSlot::Slot1 => 0,
            ActionSlot::Slot2 => 1,
            ActionSlot::Slot3 => 2,
            ActionSlot::Slot4 => 3,
        }
    }

    pub const fn from_index(index: usize) -> Option<Self> {
        match index {
            0 => Some(ActionSlot::Slot1),
            1 => Some(ActionSlot::Slot2),
            2 => Some(ActionSlot::Slot3),
            3 => Some(ActionSlot::Slot4),
            _ => None,
        }
    }

    /// Código estável para a ponte HTML/mobile. Append-only.
    pub const fn code(self) -> u8 {
        self.index() as u8
    }
}

/// Menor unidade semântica que o jogador pode acionar.
///
/// `InputAction` não conhece tecla física nem regra de jogo. O cliente mapeia
/// teclas/botões touch para esta enum e depois para [`PlayerCommand`]. `code()`
/// é a ABI estável usada pela ponte HTML.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InputAction {
    MoveLeft,
    MoveRight,
    MoveUp,
    MoveDown,

    Slot1,
    Slot2,
    Slot3,
    Slot4,

    Six,
    Seven,

    AllocateSpeed,
    AllocateStrength,
    AllocateHealth,
    AllocateAura,

    Confirm,
    Restart,
    OpenMenu,
    OpenSkills,
}

impl InputAction {
    pub const ALL: [InputAction; 18] = [
        InputAction::MoveLeft,
        InputAction::MoveRight,
        InputAction::MoveUp,
        InputAction::MoveDown,
        InputAction::Slot1,
        InputAction::Slot2,
        InputAction::Slot3,
        InputAction::Slot4,
        InputAction::Six,
        InputAction::Seven,
        InputAction::AllocateSpeed,
        InputAction::AllocateStrength,
        InputAction::AllocateHealth,
        InputAction::AllocateAura,
        InputAction::Confirm,
        InputAction::Restart,
        InputAction::OpenMenu,
        InputAction::OpenSkills,
    ];

    pub const COUNT: usize = Self::ALL.len();

    pub const fn slot(self) -> Option<ActionSlot> {
        match self {
            InputAction::Slot1 => Some(ActionSlot::Slot1),
            InputAction::Slot2 => Some(ActionSlot::Slot2),
            InputAction::Slot3 => Some(ActionSlot::Slot3),
            InputAction::Slot4 => Some(ActionSlot::Slot4),
            _ => None,
        }
    }

    pub const fn sixseven(self) -> Option<SixSevenKey> {
        match self {
            InputAction::Six => Some(SixSevenKey::Six),
            InputAction::Seven => Some(SixSevenKey::Seven),
            _ => None,
        }
    }

    pub const fn attribute(self) -> Option<Attribute> {
        match self {
            InputAction::AllocateSpeed => Some(Attribute::Speed),
            InputAction::AllocateStrength => Some(Attribute::Strength),
            InputAction::AllocateHealth => Some(Attribute::Health),
            InputAction::AllocateAura => Some(Attribute::Aura),
            _ => None,
        }
    }

    /// É ação contínua (`held`)? As demais são edges de UI/skill.
    pub const fn is_continuous(self) -> bool {
        matches!(
            self,
            InputAction::MoveLeft
                | InputAction::MoveRight
                | InputAction::MoveUp
                | InputAction::MoveDown
                | InputAction::Slot1
                | InputAction::Slot2
                | InputAction::Slot3
                | InputAction::Slot4
        )
    }

    /// Código estável para a ponte HTML/mobile. A ordem é ABI: só acrescente
    /// variantes no fim.
    pub const fn code(self) -> u8 {
        match self {
            InputAction::MoveLeft => 0,
            InputAction::MoveRight => 1,
            InputAction::MoveUp => 2,
            InputAction::MoveDown => 3,
            InputAction::Slot1 => 4,
            InputAction::Slot2 => 5,
            InputAction::Slot3 => 6,
            InputAction::Slot4 => 7,
            InputAction::Six => 8,
            InputAction::Seven => 9,
            InputAction::AllocateSpeed => 10,
            InputAction::AllocateStrength => 11,
            InputAction::AllocateHealth => 12,
            InputAction::AllocateAura => 13,
            InputAction::Confirm => 14,
            InputAction::Restart => 15,
            InputAction::OpenMenu => 16,
            InputAction::OpenSkills => 17,
        }
    }

    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(InputAction::MoveLeft),
            1 => Some(InputAction::MoveRight),
            2 => Some(InputAction::MoveUp),
            3 => Some(InputAction::MoveDown),
            4 => Some(InputAction::Slot1),
            5 => Some(InputAction::Slot2),
            6 => Some(InputAction::Slot3),
            7 => Some(InputAction::Slot4),
            8 => Some(InputAction::Six),
            9 => Some(InputAction::Seven),
            10 => Some(InputAction::AllocateSpeed),
            11 => Some(InputAction::AllocateStrength),
            12 => Some(InputAction::AllocateHealth),
            13 => Some(InputAction::AllocateAura),
            14 => Some(InputAction::Confirm),
            15 => Some(InputAction::Restart),
            16 => Some(InputAction::OpenMenu),
            17 => Some(InputAction::OpenSkills),
            _ => None,
        }
    }
}

/// Estado de um slot de habilidade no tick.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlotInput {
    /// Edge: true apenas no tick em que o botão foi pressionado.
    pub pressed: bool,
    /// Estado contínuo: true enquanto o botão estiver segurado.
    pub held: bool,
}

impl SlotInput {
    pub const fn pressed() -> Self {
        Self {
            pressed: true,
            held: true,
        }
    }
}

/// Intenção de um jogador em um único tick.
///
/// `move_x`/`move_y` e o `held` dos slots são contínuos. `pressed`, `sixseven`,
/// `allocate`, `confirm` e `restart` são edge-triggered: o cliente envia `true`
/// apenas no tick em que a tecla foi pressionada.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerCommand {
    pub move_x: i8,
    pub move_y: i8,
    pub slots: [SlotInput; ActionSlot::COUNT],
    pub sixseven: Option<SixSevenKey>,
    pub allocate: Option<Attribute>,
    pub confirm: bool,
    pub restart: bool,
}

impl PlayerCommand {
    pub const fn none() -> Self {
        Self {
            move_x: 0,
            move_y: 0,
            slots: [SlotInput {
                pressed: false,
                held: false,
            }; ActionSlot::COUNT],
            sixseven: None,
            allocate: None,
            confirm: false,
            restart: false,
        }
    }

    /// Mantém apenas os estados contínuos. Usado quando um frame produz mais de
    /// um tick: o edge não pode ser aplicado duas vezes.
    pub fn without_edges(mut self) -> Self {
        for slot in &mut self.slots {
            slot.pressed = false;
        }
        self.sixseven = None;
        self.allocate = None;
        self.confirm = false;
        self.restart = false;
        self
    }

    pub fn slot(&self, slot: ActionSlot) -> SlotInput {
        self.slots[slot.index()]
    }

    pub fn slot_mut(&mut self, slot: ActionSlot) -> &mut SlotInput {
        &mut self.slots[slot.index()]
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

/// Eixo digital com histerese zero: os dois lados juntos param.
pub fn axis(negative: bool, positive: bool) -> i8 {
    match (negative, positive) {
        (true, false) => -1,
        (false, true) => 1,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_sets_first_player() {
        let mut command = PlayerCommand::none();
        command.move_x = 1;
        command.slots[ActionSlot::Slot1.index()].pressed = true;

        let input = TickInput::single(command);
        let command = input.command(0).unwrap();
        assert_eq!(command.move_x, 1);
        assert!(command.slot(ActionSlot::Slot1).pressed);
        assert!(input.command(1).is_none());
    }

    #[test]
    fn with_player_ignores_out_of_range() {
        let input = TickInput::default().with_player(MAX_PLAYERS, PlayerCommand::none());
        assert!(input.commands.iter().all(Option::is_none));
    }

    #[test]
    fn without_edges_keeps_held_state() {
        let mut command = PlayerCommand::none();
        command.move_x = -1;
        command.move_y = 1;
        command.slots[ActionSlot::Slot2.index()] = SlotInput {
            pressed: true,
            held: true,
        };
        command.sixseven = Some(SixSevenKey::Six);
        command.allocate = Some(Attribute::Aura);
        command.confirm = true;
        command.restart = true;

        let command = command.without_edges();

        assert_eq!(command.move_x, -1);
        assert_eq!(command.move_y, 1);
        assert!(command.slot(ActionSlot::Slot2).held);
        assert!(!command.slot(ActionSlot::Slot2).pressed);
        assert!(command.sixseven.is_none());
        assert!(command.allocate.is_none());
        assert!(!command.confirm);
        assert!(!command.restart);
    }

    #[test]
    fn input_action_codes_are_stable_and_roundtrip() {
        for (index, action) in InputAction::ALL.into_iter().enumerate() {
            assert_eq!(action.code() as usize, index);
            assert_eq!(InputAction::from_code(action.code()), Some(action));
        }
        assert_eq!(InputAction::from_code(InputAction::COUNT as u8), None);
    }

    #[test]
    fn slot_and_action_helpers_agree() {
        for slot in ActionSlot::ALL {
            assert_eq!(ActionSlot::from_index(slot.index()), Some(slot));
            let action = InputAction::ALL
                .into_iter()
                .find(|action| action.slot() == Some(slot))
                .unwrap();
            assert_eq!(action.code(), 4 + slot.code());
        }
        assert_eq!(InputAction::Six.sixseven(), Some(SixSevenKey::Six));
        assert_eq!(
            InputAction::AllocateHealth.attribute(),
            Some(Attribute::Health)
        );
        assert_eq!(InputAction::Confirm.attribute(), None);
    }

    #[test]
    fn axis_is_digital_and_cancels_opposites() {
        assert_eq!(axis(false, false), 0);
        assert_eq!(axis(true, false), -1);
        assert_eq!(axis(false, true), 1);
        assert_eq!(axis(true, true), 0);
    }
}
