//! Shout: area knockback utility (placeholder).

use crate::entities::{AttributeSet, EntityId, EntityStore, EquipmentTags};
use crate::events::GameEvent;
use crate::math::Rect;

use super::{
    Activation, EntityType, Requirements, SkillActivation, SkillAssets, SkillCategory, SkillDef,
    SkillId, SkillResult, SkillSource, SkillTiming,
};

pub const COST: f32 = 4.0;
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
    base_skill_level: 1,
    min_skill_level: 1,
    progression: &[1, 2, 3, 4, 5],
    aura_cost: COST,
    cooldown: COOLDOWN,
    timing: SkillTiming::PostPhysics,
    assets: SkillAssets {
        icon: "assets/skills/shout/icon.png",
        entity: None,
        particles: Some("assets/skills/shout/wave.png"),
    },
};

/// Visual/animation metadata co-located with the skill module.
pub mod visual {
    pub const ICON: &str = "assets/skills/shout/icon.png";
    pub const WAVE: &str = "assets/skills/shout/wave.png";
    pub const ANIM_DURATION: f32 = super::ANIM_TIME;
}

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
    let (steps, ego) = store
        .get(act.entity_id)
        .map(|entity| {
            let ctx = entity.skill_context();
            (
                SkillId::Shout.tier_steps(ctx).max(0) as f32,
                entity.attributes.ego as f32,
            )
        })
        .unwrap_or((0.0, 0.0));
    let scale = 1.0 + 0.10 * steps;
    let range = RANGE * scale + ego * 2.0 * scale;
    let damage = (DAMAGE * scale + ego * 0.5 * (1.0 + 0.25 * steps))
        .round()
        .max(1.0);
    state.anim_timer = ANIM_TIME;
    let hitbox = Rect::new(
        act.rect.x - range,
        act.rect.y - 8.0 * scale,
        act.rect.w + range * 2.0,
        act.rect.h + 16.0 * scale,
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
        let applied = target.health.damage(damage);
        if applied > 0.0 {
            target.body.vel.x = dir * KNOCKBACK * scale;
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
