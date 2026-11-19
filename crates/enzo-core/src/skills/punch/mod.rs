//! Simple physical punch attack.

use crate::entities::{AttributeSet, EntityId, EntityStore, EquipmentTags};
use crate::events::GameEvent;
use crate::math::Rect;

use super::{
    Activation, EntityType, Requirements, SkillActivation, SkillAssets, SkillCategory, SkillDef,
    SkillId, SkillResult, SkillSource, SkillTiming,
};

pub const MELEE_RANGE: f32 = 34.0;
pub const MELEE_HEIGHT: f32 = 26.0;
pub const PUNCH_ANIM_TIME: f32 = 0.18;
pub const PUNCH_COOLDOWN: f32 = 0.30;
pub const PUNCH_VARIANTS: u8 = 3;

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
    base_skill_level: 1,
    min_skill_level: 1,
    progression: &[1, 2, 3, 4, 5],
    aura_cost: 0.0,
    cooldown: PUNCH_COOLDOWN,
    timing: SkillTiming::PostPhysics,
    assets: SkillAssets {
        icon: "assets/skills/melee/icon.png",
        entity: None,
        particles: Some("assets/skills/melee/impact.png"),
    },
};

/// Visual/animation metadata co-located with the skill module.
pub mod visual {
    pub const ICON: &str = "assets/skills/melee/icon.png";
    pub const IMPACT: &str = "assets/skills/melee/impact.png";
    pub const PUNCH_VARIANTS: u8 = super::PUNCH_VARIANTS;
    pub const SWING_DURATION: f32 = super::PUNCH_ANIM_TIME;
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct State {
    /// Animation variant/frame index (1..=PUNCH_VARIANTS), chosen randomly per punch.
    pub variant: u8,
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
                SkillId::Melee.tier_steps(ctx).max(0) as f32,
                entity.attributes.ego as f32,
            )
        })
        .unwrap_or((0.0, 0.0));
    let scale = 1.0 + 0.10 * steps;
    let ego_damage_bonus = ego * 0.5 * (1.0 + 0.25 * steps);

    let variant = ((act.tick as u32)
        .wrapping_mul(1664525)
        .wrapping_add(act.entity_id.0)
        .wrapping_mul(1013904223)
        % PUNCH_VARIANTS as u32
        + 1) as u8;

    state.variant = variant;
    state.anim_timer = PUNCH_ANIM_TIME;

    let facing = if act.facing >= 0 { 1.0 } else { -1.0 };
    let range = MELEE_RANGE * scale;
    let height = MELEE_HEIGHT * scale;
    let hitbox = if facing > 0.0 {
        Rect::new(
            act.rect.right(),
            act.rect.y + (act.rect.h - height) * 0.5,
            range,
            height,
        )
    } else {
        Rect::new(
            act.rect.x - range,
            act.rect.y + (act.rect.h - height) * 0.5,
            range,
            height,
        )
    };

    let damage = (act.stats.melee_damage() * scale + ego_damage_bonus)
        .round()
        .max(1.0);
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
        events.push(GameEvent::SkillHit {
            attacker: act.entity_id,
            target: target_id,
            skill: SkillId::Melee,
            damage: applied,
            combo: 1,
        });
    }

    events.push(GameEvent::SkillActivated {
        player: act.entity_id,
        slot: act.slot,
        skill: SkillId::Melee,
    });
    SkillResult::Started
}
