//! Combo físico de três golpes.

use crate::entities::{AttributeSet, EntityId, EntityStore, EquipmentTags};
use crate::events::GameEvent;
use crate::math::Rect;

use super::{
    Activation, EntityType, Requirements, SkillActivation, SkillAssets, SkillCategory, SkillDef,
    SkillId, SkillResult, SkillSource, SkillTiming,
};

pub const MELEE_RANGE: f32 = 34.0;
pub const MELEE_HEIGHT: f32 = 26.0;
pub const MELEE_WINDOW: f32 = 0.5;
pub const MELEE_ANIM_TIME: f32 = 0.18;
pub const KNOCKBACK_SPEED: f32 = 220.0;
pub const KNOCKBACK_STUN: f32 = 0.18;

pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NONE,
);

pub static DEF: SkillDef = SkillDef {
    id: SkillId::Melee,
    category: SkillCategory::Attack,
    source: SkillSource::Innate,
    activation: Activation::Press,
    requirements: REQUIREMENTS,
    aura_cost: 0.0,
    cooldown: 0.0,
    timing: SkillTiming::PostPhysics,
    assets: SkillAssets {
        icon: "assets/skills/melee/icon.png",
        entity: None,
        particles: Some("assets/skills/melee/impact.png"),
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    pub step: u8,
    pub window: f32,
    pub anim_timer: f32,
}

impl State {
    pub fn tick(&mut self, dt: f32) {
        self.window = (self.window - dt).max(0.0);
        if self.window <= 0.0 {
            self.step = 0;
        }
        self.anim_timer = (self.anim_timer - dt).max(0.0);
    }
}

pub fn activate(
    state: &mut State,
    act: SkillActivation<'_>,
    store: &mut EntityStore,
    events: &mut Vec<GameEvent>,
) -> SkillResult {
    if !act.input.pressed {
        return SkillResult::Idle;
    }

    let step = if state.window > 0.0 {
        (state.step % 3) + 1
    } else {
        1
    };

    state.step = step;
    state.window = MELEE_WINDOW;
    state.anim_timer = MELEE_ANIM_TIME;

    let facing = if act.facing >= 0 { 1.0 } else { -1.0 };
    let hitbox = if facing > 0.0 {
        Rect::new(
            act.rect.right(),
            act.rect.y + (act.rect.h - MELEE_HEIGHT) * 0.5,
            MELEE_RANGE,
            MELEE_HEIGHT,
        )
    } else {
        Rect::new(
            act.rect.x - MELEE_RANGE,
            act.rect.y + (act.rect.h - MELEE_HEIGHT) * 0.5,
            MELEE_RANGE,
            MELEE_HEIGHT,
        )
    };

    let damage = act.stats.melee_damage((step - 1) as usize);
    let targets: Vec<EntityId> = store
        .hostiles_in_rect(hitbox, act.team)
        .into_iter()
        .filter(|id| *id != act.entity_id)
        .filter(|id| store.get(*id).is_some_and(|entity| entity.is_actor()))
        .collect();

    for target_id in targets {
        let Some(target) = store.get_mut(target_id) else {
            continue;
        };
        let applied = target.health.damage(damage);
        if applied <= 0.0 {
            continue;
        }
        if let Some(enemy) = target.enemy_mut() {
            enemy.stun = KNOCKBACK_STUN;
        }
        if step == 3 {
            target.body.vel.x = facing * KNOCKBACK_SPEED;
        }
        events.push(GameEvent::SkillHit {
            attacker: act.entity_id,
            target: target_id,
            skill: SkillId::Melee,
            damage: applied,
            combo: step,
        });
    }

    events.push(GameEvent::SkillActivated {
        player: act.entity_id,
        slot: act.slot,
        skill: SkillId::Melee,
    });
    SkillResult::Started
}
