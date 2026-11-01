//! Snapshot serializável do mundo.
//!
//! O snapshot é o contrato entre o core e o render/save/rede. Ele contém apenas
//! estado derivado; nenhuma regra de jogo vive aqui. Os tiles estáticos do mapa
//! ficam em [`crate::map::Level`] e são lidos pelo render via `World::level()`.

use crate::entities::{
    Attributes, EnemySpecies, Entity, EntityId, EntityKind, Equipment, ItemCategory, ItemKind,
    ItemUsage, PlayerId,
};
use crate::math::{Rect, Vec2};
use crate::progression;
use crate::world::GamePhase;

pub const SNAPSHOT_VERSION: u16 = 1;

/// Fotografia do mundo em um tick. Não contém lógica.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorldSnapshot {
    pub version: u16,
    pub tick: u64,
    pub phase: GamePhase,
    /// Tempo restante de respawn quando a fase é `Dead`; zero nas demais.
    pub phase_timer: f32,
    pub entities: Vec<EntitySnapshot>,
    pub level_width: f32,
    pub level_height: f32,
    pub flag: Option<Rect>,
    pub camera_hint: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntitySnapshot {
    pub id: EntityId,
    pub kind: SnapshotKind,
    pub pos: Vec2,
    pub size: Vec2,
    pub vel: Vec2,
    pub facing: i8,
    pub on_ground: bool,
    pub health: f32,
    pub max_health: f32,
    pub aura: f32,
    pub max_aura: f32,
    pub combo: u32,
    pub running: bool,
    pub dashing: bool,
    pub iframes: f32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SnapshotKind {
    Player(PlayerSnapshot),
    Enemy(EnemySnapshot),
    Projectile(ProjectileSnapshot),
    Item(ItemSnapshot),
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlayerSnapshot {
    pub player_id: PlayerId,
    pub level: u32,
    pub xp: f32,
    pub xp_to_next: f32,
    pub points: u32,
    pub attributes: Attributes,
    pub equipment: Equipment,
    pub pending_level_up: bool,
    pub farming: bool,
    pub combo_window: f32,
    pub melee_step: u8,
    pub melee_timer: f32,
    pub mogar_cooldown: f32,
    pub respawn: Vec2,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnemySnapshot {
    pub species: EnemySpecies,
    pub patrol_dir: i8,
    pub stun: f32,
    pub xp_reward: f32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ProjectileSnapshot {
    pub owner: EntityId,
    pub damage: f32,
    pub lifetime: f32,
}

#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ItemSnapshot {
    pub kind: ItemKind,
    pub usage: ItemUsage,
    pub category: ItemCategory,
}

impl EntitySnapshot {
    pub fn from_entity(entity: &Entity) -> Self {
        let kind = match &entity.kind {
            EntityKind::Player(data) => SnapshotKind::Player(PlayerSnapshot {
                player_id: data.player_id,
                level: data.level,
                xp: data.xp,
                xp_to_next: progression::xp_to_next(data.level),
                points: data.points,
                attributes: entity.attributes,
                equipment: entity.equipment,
                pending_level_up: data.pending_level_up,
                farming: entity.movement.sixseven.is_active(),
                combo_window: entity.movement.sixseven.window,
                melee_step: entity.combat.melee.step,
                melee_timer: entity.combat.melee.anim_timer,
                mogar_cooldown: entity.combat.mogar.cooldown,
                respawn: data.respawn,
            }),
            EntityKind::Enemy(data) => SnapshotKind::Enemy(EnemySnapshot {
                species: data.species,
                patrol_dir: entity.ai.map(|ai| ai.patrol_dir()).unwrap_or(0),
                stun: data.stun,
                xp_reward: data.xp_reward,
            }),
            EntityKind::Projectile(data) => SnapshotKind::Projectile(ProjectileSnapshot {
                owner: data.owner,
                damage: data.damage,
                lifetime: data.lifetime,
            }),
            EntityKind::Item(data) => SnapshotKind::Item(ItemSnapshot {
                kind: data.kind,
                usage: data.usage(),
                category: data.category,
            }),
        };

        let (aura, max_aura, combo) = match &entity.kind {
            EntityKind::Player(data) => (
                data.aura,
                entity.stats.max_aura,
                entity.movement.sixseven.combo,
            ),
            _ => (0.0, entity.stats.max_aura, 0),
        };

        Self {
            id: entity.id,
            kind,
            pos: entity.body.pos,
            size: entity.body.size,
            vel: entity.body.vel,
            facing: entity.body.facing,
            on_ground: entity.body.on_ground,
            health: entity.health.current,
            max_health: entity.stats.max_health,
            aura,
            max_aura,
            combo,
            running: entity.movement.walk.running,
            dashing: entity.movement.is_dashing(),
            iframes: entity.health.iframes,
        }
    }
}
