//! Nutella Shot: equipment-granted projectile placeholder.

use crate::entities::{AttributeSet, EntitySpec, EntityStore, EquipmentTags};
use crate::events::GameEvent;
use crate::math::Vec2;

use super::{
    Activation, EntityType, Requirements, SkillActivation, SkillAssets, SkillCategory, SkillDef,
    SkillId, SkillResult, SkillSource, SkillTiming,
};

pub const COST: f32 = 0.0;
pub const COOLDOWN: f32 = 0.25;
pub const SIZE: Vec2 = Vec2::new(16.0, 10.0);
pub const SPEED: f32 = 420.0;
pub const LIFETIME: f32 = 1.2;
pub const DAMAGE: f32 = 14.0;

pub const REQUIREMENTS: Requirements = Requirements::new(
    &[EntityType::Playable],
    1,
    AttributeSet::ZERO,
    EquipmentTags::NUTELLA_GUN,
);

pub static DEF: SkillDef = SkillDef {
    id: SkillId::NutellaShot,
    category: SkillCategory::Attack,
    source: SkillSource::Equipment,
    activation: Activation::Press,
    requirements: REQUIREMENTS,
    aura_cost: COST,
    cooldown: COOLDOWN,
    timing: SkillTiming::PostPhysics,
    assets: SkillAssets {
        icon: "assets/skills/nutella_shot/icon.png",
        entity: Some("assets/skills/nutella_shot/projectile.png"),
        particles: Some("assets/skills/nutella_shot/splash.png"),
    },
};

/// Visual/animation metadata co-located with the skill module.
pub mod visual {
    pub const ICON: &str = "assets/skills/nutella_shot/icon.png";
    pub const PROJECTILE: &str = "assets/skills/nutella_shot/projectile.png";
    pub const SPLASH: &str = "assets/skills/nutella_shot/splash.png";
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
    let facing = if act.facing >= 0 { 1 } else { -1 };
    let pos = Vec2::new(
        if facing > 0 {
            act.rect.right() + 2.0
        } else {
            act.rect.x - SIZE.x - 2.0
        },
        act.rect.y + (act.rect.h - SIZE.y) * 0.5,
    );
    let projectile = store.spawn(EntitySpec::projectile(
        SkillId::NutellaShot,
        act.entity_id,
        pos,
        facing,
        DAMAGE,
        act.team,
        SIZE,
        SPEED,
        LIFETIME,
    ));
    events.push(GameEvent::SkillProjectileSpawned {
        owner: act.entity_id,
        skill: SkillId::NutellaShot,
        projectile,
    });
    events.push(GameEvent::SkillActivated {
        player: act.entity_id,
        slot: act.slot,
        skill: SkillId::NutellaShot,
    });
    SkillResult::Started
}
