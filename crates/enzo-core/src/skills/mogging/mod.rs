//! Mogging: projétil que gasta aura e atravessa inimigos.

use crate::entities::{AttributeSet, EntityId, EntitySpec, EntityStore, EquipmentTags};
use crate::events::GameEvent;
use crate::map::Level;
use crate::math::{Rect, Vec2};

use super::{
    Activation, EntityType, Requirements, SkillActivation, SkillAssets, SkillCategory, SkillDef,
    SkillId, SkillResult, SkillSource, SkillTiming,
};

pub const MOGGING_COST: f32 = 25.0;
pub const MOGGING_COOLDOWN: f32 = 0.35;
pub const MOGGING_SIZE: Vec2 = Vec2::new(24.0, 12.0);
pub const MOGGING_SPEED: f32 = 620.0;
pub const MOGGING_LIFETIME: f32 = 1.4;

pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NONE,
);

pub static DEF: SkillDef = SkillDef {
    id: SkillId::Mogging,
    category: SkillCategory::Attack,
    source: SkillSource::Innate,
    activation: Activation::Press,
    requirements: REQUIREMENTS,
    aura_cost: MOGGING_COST,
    cooldown: MOGGING_COOLDOWN,
    timing: SkillTiming::PostPhysics,
    assets: SkillAssets {
        icon: "assets/skills/mogging/icon.png",
        entity: Some("assets/skills/mogging/projectile.png"),
        particles: Some("assets/skills/mogging/impact.png"),
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State;

impl State {
    pub fn tick(&mut self, _dt: f32) {}
}

pub fn activate(
    _state: &mut State,
    act: SkillActivation<'_>,
    store: &mut EntityStore,
    events: &mut Vec<GameEvent>,
) -> SkillResult {
    if !act.input.pressed {
        return SkillResult::Idle;
    }

    let facing = if act.facing >= 0 { 1 } else { -1 };
    let pos = Vec2::new(
        if facing > 0 {
            act.rect.right() + 4.0
        } else {
            act.rect.x - MOGGING_SIZE.x - 4.0
        },
        act.rect.y + (act.rect.h - MOGGING_SIZE.y) * 0.5,
    );
    let projectile = store.spawn(EntitySpec::projectile(
        SkillId::Mogging,
        act.entity_id,
        pos,
        facing,
        act.stats.mogging_damage,
        act.team,
        MOGGING_SIZE,
        MOGGING_SPEED,
        MOGGING_LIFETIME,
    ));
    events.push(GameEvent::SkillProjectileSpawned {
        owner: act.entity_id,
        skill: SkillId::Mogging,
        projectile,
    });
    events.push(GameEvent::SkillActivated {
        player: act.entity_id,
        slot: act.slot,
        skill: SkillId::Mogging,
    });
    SkillResult::Started
}

/// Move skill projectiles, apply damage, and remove expired/collided ones.
///
/// Solid blocks consume projectiles. Destructible solids take 1 block-damage hit
/// and emit block events; the projectile is consumed on impact.
pub fn update_projectiles(
    store: &mut EntityStore,
    level: &mut Level,
    dt: f32,
    events: &mut Vec<GameEvent>,
) {
    let ids: Vec<EntityId> = store
        .iter()
        .filter(|entity| entity.is_projectile())
        .map(|entity| entity.id)
        .collect();

    for id in ids {
        let Some(entity) = store.get(id) else {
            continue;
        };
        let Some(data) = entity.projectile() else {
            continue;
        };
        let owner = data.owner;
        let skill = data.skill;
        let damage = data.damage;
        let hits = data.hits.clone();
        let lifetime = data.lifetime;
        let team = entity.team;
        let pos = entity.body.pos;
        let vel = entity.body.vel;
        let size = entity.body.size;

        let new_pos = pos + vel * dt;
        let rect = Rect::new(new_pos.x, new_pos.y, size.x, size.y);

        if level.rect_overlaps_solid(rect) {
            // Apply one damage hit to destructible blocks in the impact area.
            level.damage_solid_blocks_in_rect(rect, 1, events);
            store.despawn(id);
            events.push(GameEvent::SkillProjectileBlocked { owner, skill });
            continue;
        }

        if let Some(entity) = store.get_mut(id) {
            entity.body.pos = new_pos;
            if let Some(data) = entity.projectile_mut() {
                data.lifetime = lifetime - dt;
            }
        }

        let targets: Vec<EntityId> = store
            .hostiles_in_rect(rect, team)
            .into_iter()
            .filter(|target| *target != id)
            .filter(|target| store.get(*target).is_some_and(|entity| entity.is_actor()))
            .filter(|target| !hits.contains(target))
            .collect();

        for target_id in targets {
            let applied = store
                .get_mut(target_id)
                .map(|target| target.health.damage(damage))
                .unwrap_or(0.0);
            if applied <= 0.0 {
                continue;
            }
            if let Some(data) = store.get_mut(id).and_then(|entity| entity.projectile_mut()) {
                data.hits.push(target_id);
            }
            events.push(GameEvent::SkillHit {
                attacker: owner,
                target: target_id,
                skill,
                damage: applied,
                combo: 0,
            });
        }

        let expired = store
            .get(id)
            .and_then(|entity| entity.projectile())
            .is_some_and(|data| data.lifetime <= 0.0);
        if expired {
            store.despawn(id);
        }
    }
}
