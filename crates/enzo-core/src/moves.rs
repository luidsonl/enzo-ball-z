//! Catálogo e atribuição de movimentos.
//!
//! Todo movimento — inerente (`Walk`, `Jump`, `Dash`, `Fly`) ou de ataque
//! (`Melee`, `Mogar`) — declara seus [`Requirements`]. Uma entidade recebe um
//! [`MoveSet`] com apenas os movimentos que ela pode usar.
//!
//! A validação é feita no core: o cliente não decide se o Enzo pode voar ou se
//! um inimigo pode dar Mogar. Novos movimentos entram como uma variante de
//! [`MoveId`], um componente no diretório correspondente (`movements/` ou
//! `attacks/`) e uma entrada em [`MoveId::requirements`].

use crate::attacks::{melee, mogar};
use crate::entity::Attributes;
use crate::movements::{dash, fly, jump, walk};

/// Tipo de entidade usado para validar requisitos de atribuição.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EntityType {
    Player,
    Enemy,
    Projectile,
}

/// Identidade estável de um movimento. A ordem é usada pelo [`MoveSet`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MoveId {
    Walk,
    Jump,
    Dash,
    Fly,
    Melee,
    Mogar,
}

impl MoveId {
    pub const ALL: [MoveId; 6] = [
        MoveId::Walk,
        MoveId::Jump,
        MoveId::Dash,
        MoveId::Fly,
        MoveId::Melee,
        MoveId::Mogar,
    ];

    pub const MOVEMENTS: [MoveId; 4] = [MoveId::Walk, MoveId::Jump, MoveId::Dash, MoveId::Fly];

    pub const ATTACKS: [MoveId; 2] = [MoveId::Melee, MoveId::Mogar];

    pub fn category(self) -> MoveCategory {
        match self {
            MoveId::Walk | MoveId::Jump | MoveId::Dash | MoveId::Fly => MoveCategory::Movement,
            MoveId::Melee | MoveId::Mogar => MoveCategory::Attack,
        }
    }

    /// Requisitos declarados pelo próprio movimento.
    pub fn requirements(self) -> Requirements {
        match self {
            MoveId::Walk => walk::REQUIREMENTS,
            MoveId::Jump => jump::REQUIREMENTS,
            MoveId::Dash => dash::REQUIREMENTS,
            MoveId::Fly => fly::REQUIREMENTS,
            MoveId::Melee => melee::REQUIREMENTS,
            MoveId::Mogar => mogar::REQUIREMENTS,
        }
    }

    fn bit(self) -> u16 {
        1u16 << (self as u8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MoveCategory {
    Movement,
    Attack,
}

/// Contexto da entidade no momento da atribuição. Nível e atributos contam;
/// posição, vida e input não.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbilityContext {
    pub entity_type: EntityType,
    pub level: u32,
    pub attributes: Attributes,
}

impl AbilityContext {
    pub const fn new(entity_type: EntityType, level: u32, attributes: Attributes) -> Self {
        Self {
            entity_type,
            level,
            attributes,
        }
    }

    pub fn player(level: u32, attributes: Attributes) -> Self {
        Self::new(EntityType::Player, level, attributes)
    }

    pub fn enemy() -> Self {
        Self::new(EntityType::Enemy, 1, Attributes::ZERO)
    }

    pub fn projectile() -> Self {
        Self::new(EntityType::Projectile, 1, Attributes::ZERO)
    }
}

/// Regras para atribuir um movimento a uma entidade.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Requirements {
    pub entity_types: &'static [EntityType],
    pub min_level: u32,
    pub min_attributes: Attributes,
}

impl Requirements {
    pub const fn new(
        entity_types: &'static [EntityType],
        min_level: u32,
        min_attributes: Attributes,
    ) -> Self {
        Self {
            entity_types,
            min_level,
            min_attributes,
        }
    }

    /// Valida o contexto inteiro e devolve o primeiro motivo de recusa.
    pub fn allows(self, context: AbilityContext) -> Result<(), AssignError> {
        if !self.entity_types.contains(&context.entity_type) {
            return Err(AssignError::WrongEntityType {
                required: self.entity_types,
                current: context.entity_type,
            });
        }
        if context.level < self.min_level {
            return Err(AssignError::LevelTooLow {
                required: self.min_level,
                current: context.level,
            });
        }
        if !context.attributes.meets(self.min_attributes) {
            return Err(AssignError::AttributesTooLow {
                required: self.min_attributes,
                current: context.attributes,
            });
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignError {
    WrongCategory,
    WrongEntityType {
        required: &'static [EntityType],
        current: EntityType,
    },
    LevelTooLow {
        required: u32,
        current: u32,
    },
    AttributesTooLow {
        required: Attributes,
        current: Attributes,
    },
}

/// Conjunto determinístico de movimentos atribuídos a uma entidade.
///
/// É um bitset para caber no snapshot e não depender de `HashSet`. A iteração
/// segue a ordem de [`MoveId::ALL`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MoveSet {
    bits: u16,
}

impl MoveSet {
    pub const EMPTY: Self = Self { bits: 0 };

    pub const fn new() -> Self {
        Self::EMPTY
    }

    pub(crate) fn from_slice(ids: &[MoveId]) -> Self {
        let mut set = Self::new();
        for id in ids {
            set.insert(*id);
        }
        set
    }

    pub fn contains(self, id: MoveId) -> bool {
        self.bits & id.bit() != 0
    }

    pub fn is_empty(self) -> bool {
        self.bits == 0
    }

    pub fn len(self) -> u32 {
        self.bits.count_ones()
    }

    pub fn iter(self) -> impl Iterator<Item = MoveId> {
        MoveId::ALL.into_iter().filter(move |id| self.contains(*id))
    }

    /// Atribui o movimento somente se todos os requisitos forem cumpridos.
    /// Reatribuir é idempotente.
    pub fn try_assign(&mut self, id: MoveId, context: AbilityContext) -> Result<(), AssignError> {
        id.requirements().allows(context)?;
        self.insert(id);
        Ok(())
    }

    pub fn remove(&mut self, id: MoveId) -> bool {
        let had = self.contains(id);
        self.bits &= !id.bit();
        had
    }

    pub(crate) fn insert(&mut self, id: MoveId) {
        self.bits |= id.bit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn player_at(level: u32, attributes: Attributes) -> AbilityContext {
        AbilityContext::player(level, attributes)
    }

    #[test]
    fn move_ids_are_split_by_category() {
        assert_eq!(MoveId::Walk.category(), MoveCategory::Movement);
        assert_eq!(MoveId::Fly.category(), MoveCategory::Movement);
        assert_eq!(MoveId::Melee.category(), MoveCategory::Attack);
        assert_eq!(MoveId::Mogar.category(), MoveCategory::Attack);
    }

    #[test]
    fn default_player_set_is_valid() {
        let context = player_at(1, Attributes::ZERO);
        for id in [
            MoveId::Walk,
            MoveId::Jump,
            MoveId::Dash,
            MoveId::Melee,
            MoveId::Mogar,
        ] {
            assert!(id.requirements().allows(context).is_ok(), "{id:?}");
        }
        assert!(MoveId::Fly.requirements().allows(context).is_err());
    }

    #[test]
    fn walk_accepts_player_and_enemy_only() {
        let walk = MoveId::Walk.requirements();
        assert!(walk.allows(AbilityContext::enemy()).is_ok());
        assert!(walk.allows(player_at(1, Attributes::ZERO)).is_ok());
        assert_eq!(
            walk.allows(AbilityContext::projectile()),
            Err(AssignError::WrongEntityType {
                required: walk.entity_types,
                current: EntityType::Projectile,
            })
        );
    }

    #[test]
    fn fly_requires_level_and_aura() {
        let fly = MoveId::Fly.requirements();
        assert_eq!(
            fly.allows(player_at(1, Attributes::ZERO)),
            Err(AssignError::LevelTooLow {
                required: 5,
                current: 1,
            })
        );
        assert_eq!(
            fly.allows(player_at(5, Attributes::ZERO)),
            Err(AssignError::AttributesTooLow {
                required: Attributes {
                    aura: 5,
                    ..Attributes::ZERO
                },
                current: Attributes::ZERO,
            })
        );
        let attrs = Attributes {
            aura: 5,
            ..Attributes::ZERO
        };
        assert!(fly.allows(player_at(5, attrs)).is_ok());
    }

    #[test]
    fn moveset_is_deterministic_and_idempotent() {
        let context = player_at(1, Attributes::ZERO);
        let mut set = MoveSet::new();
        assert!(set.try_assign(MoveId::Walk, context).is_ok());
        assert!(set.try_assign(MoveId::Walk, context).is_ok());
        assert!(set.try_assign(MoveId::Melee, context).is_ok());

        assert_eq!(
            set.iter().collect::<Vec<_>>(),
            vec![MoveId::Walk, MoveId::Melee]
        );
        assert_eq!(set.len(), 2);
        assert!(set.remove(MoveId::Walk));
        assert!(!set.remove(MoveId::Walk));
        assert!(!set.contains(MoveId::Walk));
    }

    #[test]
    fn rejected_assignment_does_not_insert() {
        let mut set = MoveSet::new();
        assert!(set
            .try_assign(MoveId::Fly, player_at(1, Attributes::ZERO))
            .is_err());
        assert!(!set.contains(MoveId::Fly));
    }
}
