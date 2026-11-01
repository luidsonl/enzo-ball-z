//! Eventos do tick. São dados, não estado persistente.

use crate::entities::{EntityId, PlayerId};
use crate::input::Attribute;

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    MeleeHit {
        attacker: EntityId,
        target: EntityId,
        damage: f32,
        combo: u8,
    },
    MogarFired {
        owner: EntityId,
        projectile: EntityId,
    },
    MogarNoAura {
        owner: EntityId,
    },
    MogarHit {
        owner: EntityId,
        target: EntityId,
        damage: f32,
    },
    MogarBlocked {
        owner: EntityId,
    },
    PlayerDamaged {
        player: EntityId,
        source: EntityId,
        damage: f32,
    },
    EnemyKilled {
        enemy: EntityId,
        xp: f32,
    },
    LevelUp {
        player: EntityId,
        level: u32,
    },
    PlayerDied {
        player: EntityId,
    },
    Victory {
        player: EntityId,
    },
    AttributeAllocated {
        player: EntityId,
        attribute: Attribute,
    },
    PhaseRestarted {
        player: PlayerId,
    },
    SaveRequested,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TickOutput {
    pub events: Vec<GameEvent>,
}

impl TickOutput {
    pub fn new(events: Vec<GameEvent>) -> Self {
        Self { events }
    }
}
