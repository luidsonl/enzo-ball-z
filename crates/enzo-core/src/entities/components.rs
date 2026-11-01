//! Componentes compartilhados por todas as entidades.
//!
//! Jogável, Capanga e Mogar usam os mesmos componentes de base. O que muda é o
//! [`crate::entities::EntityKind`], o controlador, o time e os componentes
//! opcionais (como IA e equipamento).

use crate::math::{Rect, Vec2};

/// Quantidade máxima de times que o core consegue representar.
pub const MAX_TEAMS: usize = 8;

/// Identidade de um time. Times não são facções hardcoded: a hostilidade é
/// definida por [`TeamRelations`] e pode ser trocada sem mudar as entidades.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamId(pub u8);

impl TeamId {
    pub const PLAYERS: Self = Self(0);
    pub const ENEMIES: Self = Self(1);
    pub const NEUTRAL: Self = Self(2);

    pub const fn new(id: u8) -> Self {
        Self(id)
    }

    pub fn index(self) -> Option<usize> {
        let index = self.0 as usize;
        (index < MAX_TEAMS).then_some(index)
    }
}

/// Relações de hostilidade entre times. A relação é simétrica no MVP: se A é
/// hostil a B, B também é hostil a A.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TeamRelations {
    hostile: [u16; MAX_TEAMS],
}

impl TeamRelations {
    pub const fn new() -> Self {
        Self {
            hostile: [0; MAX_TEAMS],
        }
    }

    /// Marca dois times como hostis nos dois sentidos.
    pub fn set_hostile(&mut self, a: TeamId, b: TeamId) {
        let (Some(a_index), Some(b_index)) = (a.index(), b.index()) else {
            return;
        };
        self.hostile[a_index] |= 1 << b_index;
        self.hostile[b_index] |= 1 << a_index;
    }

    pub fn is_hostile(&self, a: TeamId, b: TeamId) -> bool {
        let (Some(a_index), Some(b_index)) = (a.index(), b.index()) else {
            return false;
        };
        self.hostile[a_index] & (1 << b_index) != 0
    }
}

impl Default for TeamRelations {
    fn default() -> Self {
        let mut relations = Self::new();
        relations.set_hostile(TeamId::PLAYERS, TeamId::ENEMIES);
        relations
    }
}

/// Corpo físico comum a todas as entidades.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: Vec2,
    pub on_ground: bool,
    pub facing: i8,
}

impl Body {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            size,
            on_ground: false,
            facing: 1,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }

    pub fn center(&self) -> Vec2 {
        self.rect().center()
    }

    pub fn bottom(&self) -> f32 {
        self.pos.y + self.size.y
    }

    pub fn right(&self) -> f32 {
        self.pos.x + self.size.x
    }
}

/// Vida atual e i-frames.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub iframes: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self {
            current: max,
            iframes: 0.0,
        }
    }

    pub fn is_alive(self) -> bool {
        self.current > 0.0
    }

    /// Aplica dano e retorna o valor efetivamente descontado.
    pub fn damage(&mut self, amount: f32) -> f32 {
        let applied = amount.max(0.0).min(self.current);
        self.current -= applied;
        applied
    }

    pub fn heal(&mut self, amount: f32, max: f32) {
        self.current = (self.current + amount.max(0.0)).min(max);
    }

    pub fn tick(&mut self, dt: f32) {
        self.iframes = (self.iframes - dt).max(0.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn team_relations_are_symmetric_and_configurable() {
        let mut relations = TeamRelations::default();
        assert!(relations.is_hostile(TeamId::PLAYERS, TeamId::ENEMIES));
        assert!(relations.is_hostile(TeamId::ENEMIES, TeamId::PLAYERS));
        assert!(!relations.is_hostile(TeamId::PLAYERS, TeamId::PLAYERS));
        assert!(!relations.is_hostile(TeamId::PLAYERS, TeamId::NEUTRAL));

        relations.set_hostile(TeamId::PLAYERS, TeamId::NEUTRAL);
        assert!(relations.is_hostile(TeamId::NEUTRAL, TeamId::PLAYERS));
    }

    #[test]
    fn health_damage_and_heal_clamp() {
        let mut health = Health::full(30.0);
        assert_eq!(health.damage(10.0), 10.0);
        assert_eq!(health.current, 20.0);
        assert_eq!(health.damage(100.0), 20.0);
        assert_eq!(health.current, 0.0);
        assert!(!health.is_alive());
        health.heal(50.0, 30.0);
        assert_eq!(health.current, 30.0);
    }
}
