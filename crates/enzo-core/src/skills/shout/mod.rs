//! Grito: utilidade de empurrão em área (placeholder).

use crate::entities::{AttributeSet, EntityId, EntityStore, EquipmentTags};
use crate::events::GameEvent;
use crate::math::Rect;

use super::{
    Activation, EntityType, Requirements, SkillActivation, SkillAssets, SkillCategory, SkillDef,
    SkillId, SkillResult, SkillSource, SkillTiming,
};

pub const COST: f32 = 10.0;
pub const COOLDOWN: f32 = 2.0;
pub const RANGE: f32 = 72.0;
pub const DAMAGE: f32 = 4.0;
pub const KNOCKBACK: f32 = 260.0;
pub const ANIM_TIME: f32 = 0.25;

pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NONE,
);

pub static DEF: SkillDef = SkillDef {
    id: SkillId::Shout,
    category: SkillCategory::Utility,
    source: SkillSource::Innate,
    activation: Activation::Press,
    requirements: REQUIREMENTS,
    aura_cost: COST,
    cooldown: COOLDOWN,
    timing: SkillTiming::PostPhysics,
    assets: SkillAssets {
        icon: "assets/skills/shout/icon.png",
        entity: None,
        particles: Some("assets/skills/shout/wave.png"),
    },
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    pub anim_timer: f32,
}

impl State {
    pub fn tick(&mut self, dt: f32) {
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
    state.anim_timer = ANIM_TIME;
    let hitbox = Rect::new(
        act.rect.x - RANGE,
        act.rect.y - 8.0,
        act.rect.w + RANGE * 2.0,
        act.rect.h + 16.0,
    );
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
        let dir = if target.center().x >= act.rect.center().x {
            1.0
        } else {
            -1.0
        };
        let applied = target.health.damage(DAMAGE);
        if applied > 0.0 {
            target.body.vel.x = dir * KNOCKBACK;
            target.body.vel.y = -120.0;
            events.push(GameEvent::SkillHit {
                attacker: act.entity_id,
                target: target_id,
                skill: SkillId::Shout,
                damage: applied,
                combo: 0,
            });
        }
    }
    events.push(GameEvent::SkillActivated {
        player: act.entity_id,
        slot: act.slot,
        skill: SkillId::Shout,
    });
    SkillResult::Started
}
