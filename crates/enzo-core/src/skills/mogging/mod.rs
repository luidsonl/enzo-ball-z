//! Mogging: an aura-consuming projectile that pierces enemies.

use crate::entities::{AttributeSet, EntityId, EntitySpec, EntityStore, EquipmentTags};
use crate::events::GameEvent;
use crate::map::Level;
use crate::math::{Rect, Vec2};

use super::{
    Activation, EntityType, Requirements, SkillActivation, SkillAssets, SkillCategory, SkillDef,
    SkillId, SkillResult, SkillSource, SkillTiming,
};

pub const MOGGING_COST: f32 = 3.0;
pub const MOGGING_COOLDOWN: f32 = 0.35;
pub const MOGGING_SIZE: Vec2 = Vec2::new(24.0, 12.0);
pub const MOGGING_SPEED: f32 = 620.0;
pub const MOGGING_LIFETIME: f32 = 1.4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TierParams {
    pub projectiles: u8,
    pub damage_bonus_ego: f32,
    pub speed: f32,
    pub lifetime: f32,
    pub aura_cost: f32,
    pub cooldown: f32,
}

pub fn tier_params(tier: u8) -> TierParams {
    match tier {
        0 | 1 => TierParams {
            projectiles: 1,
            damage_bonus_ego: 0.0,
            speed: MOGGING_SPEED,
            lifetime: MOGGING_LIFETIME,
            aura_cost: 3.0,
            cooldown: 0.35,
        },
        3 => TierParams {
            projectiles: 1,
            damage_bonus_ego: 1.0, // +ego extra (total 2+aura+2*ego)
            speed: 680.0,
            lifetime: 1.7,
            aura_cost: 3.0,
            cooldown: 0.30,
        },
        _ => TierParams {
            // tier 5+
            projectiles: 3,
            damage_bonus_ego: 2.0, // +2*ego extra (total 2+aura+3*ego)
            speed: 700.0,
            lifetime: 1.8,
            aura_cost: 4.0,
            cooldown: 0.35,
        },
    }
}

pub const REQUIREMENTS: Requirements = Requirements {
    entity_types: &[EntityType::Playable],
    min_level: 1,
    min_skill_level: 1,
    min_attributes: AttributeSet::ZERO,
    required_equipment: EquipmentTags::NONE,
};

pub static DEF: SkillDef = SkillDef {
    id: SkillId::Mogging,
    category: SkillCategory::Attack,
    source: SkillSource::Innate,
    activation: Activation::Press,
    requirements: REQUIREMENTS,
    base_skill_level: 0,
    min_skill_level: 1,
    progression: &[1, 3, 5],
    aura_cost: MOGGING_COST,
    cooldown: MOGGING_COOLDOWN,
    timing: SkillTiming::PostPhysics,
    assets: SkillAssets {
        icon: "assets/skills/mogging/icon.png",
        entity: Some("assets/skills/mogging/projectile.png"),
        particles: Some("assets/skills/mogging/impact.png"),
    },
};

/// Visual/animation metadata co-located with the skill module.
pub mod visual {
    pub const ICON: &str = "assets/skills/mogging/icon.png";
    pub const PROJECTILE: &str = "assets/skills/mogging/projectile.png";
    pub const IMPACT: &str = "assets/skills/mogging/impact.png";
}

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

    let (tier, ego, base_damage) = store
        .get(act.entity_id)
        .map(|entity| {
            let ctx = entity.skill_context();
            let tier = SkillId::Mogging.active_tier(ctx).unwrap_or(1);
            let ego = entity.attributes.ego as f32;
            (tier, ego, act.stats.mogging_damage)
        })
        .unwrap_or((1, 0.0, act.stats.mogging_damage));
    let params = tier_params(tier);
    let damage = base_damage + ego * params.damage_bonus_ego;

    let facing = if act.facing >= 0 { 1 } else { -1 };
    let base_x = if facing > 0 {
        act.rect.right() + 4.0
    } else {
        act.rect.x - MOGGING_SIZE.x - 4.0
    };
    let base_y = act.rect.y + (act.rect.h - MOGGING_SIZE.y) * 0.5;

    let spreads: Vec<f32> = match params.projectiles {
        3 => vec![-120.0, 0.0, 120.0],
        _ => vec![0.0],
    };
    for vy in spreads {
        let projectile = store.spawn(
            EntitySpec::projectile(
                SkillId::Mogging,
                act.entity_id,
                Vec2::new(base_x, base_y),
                facing,
                damage,
                act.team,
                MOGGING_SIZE,
                params.speed,
                params.lifetime,
            )
            .with_velocity(Vec2::new(facing as f32 * params.speed, vy)),
        );
        events.push(GameEvent::SkillProjectileSpawned {
            owner: act.entity_id,
            skill: SkillId::Mogging,
            projectile,
        });
    }
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
