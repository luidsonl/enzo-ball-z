//! Eventos do tick. São dados, não estado persistente.

use crate::entities::{EntityId, PlayerId};
use crate::input::{ActionSlot, Attribute};
use crate::skills::{LoadoutChange, SkillId, SkillLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillRejectReason {
    Unavailable(SkillLock),
    Cooldown,
    NoAura,
    EmptySlot,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BlockPos {
    pub x: usize,
    pub y: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum GameEvent {
    SkillActivated {
        player: EntityId,
        slot: ActionSlot,
        skill: SkillId,
    },
    SkillRejected {
        player: EntityId,
        slot: ActionSlot,
        skill: SkillId,
        reason: SkillRejectReason,
    },
    SkillProjectileSpawned {
        owner: EntityId,
        skill: SkillId,
        projectile: EntityId,
    },
    SkillProjectileBlocked {
        owner: EntityId,
        skill: SkillId,
    },
    SkillHit {
        attacker: EntityId,
        target: EntityId,
        skill: SkillId,
        damage: f32,
        combo: u8,
    },
    BlockDamaged {
        pos: BlockPos,
        remaining_hits: u16,
    },
    BlockDestroyed {
        pos: BlockPos,
    },
    LoadoutChanged {
        player: EntityId,
        change: LoadoutChange,
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
