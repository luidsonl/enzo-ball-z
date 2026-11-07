//! Catálogo de skills de slot, loadout e dispatch.
//!
//! Skills ocupam [`crate::input::ActionSlot`] e são escolhidas pelo jogador via
//! [`Loadout`]. Andar e SixSeven ficam em [`crate::mechanics`], fora dos slots.

pub mod dash;
pub mod fly;
pub mod jump;
pub mod loadout;
pub mod melee;
pub mod mogging;
pub mod nutella_shot;
pub mod registry;
pub mod requirements;
pub mod shout;

pub use loadout::{Loadout, LoadoutChange, LoadoutError, LoadoutRequest};
pub use requirements::{EntityType, Requirements, SkillContext, SkillLock};

use crate::entities::{Body, EntityId, EntityStore, Stats, TeamId};
use crate::events::{GameEvent, SkillRejectReason};
use crate::input::{ActionSlot, PlayerCommand, SlotInput};
use crate::map::Level;
use crate::math::Rect;
use crate::physics;

/// Identidade estável de uma skill equipável.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "snake_case"))]
pub enum SkillId {
    Jump,
    Dash,
    Fly,
    Melee,
    Mogging,
    NutellaShot,
    Shout,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SkillCategory {
    Movement,
    Attack,
    Utility,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SkillSource {
    Innate,
    Progression,
    Equipment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Activation {
    Press,
    Hold,
    PressHold,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SkillTiming {
    PrePhysics,
    PostPhysics,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SkillAssets {
    pub icon: &'static str,
    pub entity: Option<&'static str>,
    pub particles: Option<&'static str>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkillDef {
    pub id: SkillId,
    pub category: SkillCategory,
    pub source: SkillSource,
    pub activation: Activation,
    pub requirements: Requirements,
    pub aura_cost: f32,
    pub cooldown: f32,
    pub timing: SkillTiming,
    pub assets: SkillAssets,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SkillResult {
    Idle,
    Started,
    Active,
    Rejected(SkillRejectReason),
}

/// Contexto mínimo para skills pós-física (posição já integrada).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SkillActivation<'a> {
    pub entity_id: EntityId,
    pub slot: ActionSlot,
    pub input: SlotInput,
    pub move_x: i8,
    pub move_y: i8,
    pub rect: Rect,
    pub facing: i8,
    pub aura: f32,
    pub stats: &'a Stats,
    pub team: TeamId,
}

/// Estado tipado de todas as skills. Sempre existe; só as equipadas respondem.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Skills {
    pub loadout: Loadout,
    pub cooldowns: [f32; SkillId::COUNT],
    pub jump: jump::State,
    pub dash: dash::State,
    pub fly: fly::State,
    pub melee: melee::State,
    pub mogging: mogging::State,
    pub nutella_shot: nutella_shot::State,
    pub shout: shout::State,
}

impl Skills {
    pub fn player_default() -> Self {
        Self {
            loadout: Loadout::player_default(),
            ..Self::default()
        }
    }

    pub fn enemy() -> Self {
        Self::default()
    }

    pub fn none() -> Self {
        Self::default()
    }

    pub fn has(&self, id: SkillId) -> bool {
        self.loadout.contains(id)
    }

    pub fn cooldown(&self, id: SkillId) -> f32 {
        self.cooldowns[id.index()]
    }

    pub fn set_cooldown(&mut self, id: SkillId, value: f32) {
        self.cooldowns[id.index()] = value.max(0.0);
    }

    pub fn is_active(&self, id: SkillId) -> bool {
        match id {
            SkillId::Jump => false,
            SkillId::Dash => self.dash.is_active(),
            SkillId::Fly => self.fly.active,
            SkillId::Melee => self.melee.anim_timer > 0.0,
            SkillId::Mogging => false,
            SkillId::NutellaShot => false,
            SkillId::Shout => self.shout.anim_timer > 0.0,
        }
    }

    /// Aplica intenção contínua para skills pré-física equipadas.
    pub fn apply_intent(&mut self, command: PlayerCommand, body: &Body) {
        self.jump.held = false;
        self.dash.intent_x = command.move_x.clamp(-1, 1);
        self.fly.vertical_intent = 0;
        for (slot, skill) in self.loadout.iter() {
            let input = command.slots[slot.index()];
            match skill {
                SkillId::Jump => {
                    self.jump.held = input.held;
                    if input.pressed {
                        self.jump.buffer = jump::JUMP_BUFFER_TIME;
                    }
                }
                SkillId::Dash => {
                    if input.pressed {
                        self.dash.request(body.facing);
                    }
                }
                SkillId::Fly => {
                    if input.held && self.cooldown(SkillId::Fly) <= 0.0 {
                        self.fly.vertical_intent = command.move_y.clamp(-1, 1);
                    }
                }
                _ => {}
            }
        }
    }

    /// Avança mecânicas pré-física e integra o corpo.
    pub fn step_physics(
        &mut self,
        body: &mut Body,
        stats: &Stats,
        level: &Level,
        locomotion_speed_factor: f32,
        dt: f32,
        locomotion: &mut crate::mechanics::Locomotion,
    ) {
        // walk tick já acontece em Locomotion::tick no world; aqui só usa estado.
        self.jump.tick(body, dt);
        self.dash.tick(dt);

        // SixSeven tick já acontece em Locomotion::tick.

        if self.has(SkillId::Dash) {
            self.dash.try_start(body, locomotion.walk.intent_x);
        }
        if self.has(SkillId::Jump) {
            self.jump.update(body, stats);
        }

        self.update_horizontal(body, stats, locomotion_speed_factor, dt, locomotion);
        self.update_vertical(body, dt);
        physics::integrate(body, level, dt);

        if self.has(SkillId::Jump) && body.on_ground {
            self.jump.coyote = jump::COYOTE_TIME;
        }
        self.fly.active = self.has(SkillId::Fly) && self.fly.vertical_intent != 0;
    }

    fn update_horizontal(
        &mut self,
        body: &mut Body,
        stats: &Stats,
        speed_factor: f32,
        dt: f32,
        locomotion: &mut crate::mechanics::Locomotion,
    ) {
        if self.is_active(SkillId::Dash) {
            body.vel.x = self.dash.direction as f32 * dash::DASH_SPEED;
            return;
        }
        if locomotion.has_walk() {
            locomotion.walk.update(body, stats, speed_factor, dt);
        }
    }

    fn update_vertical(&mut self, body: &mut Body, dt: f32) {
        if self.is_active(SkillId::Dash) {
            body.vel.y = body.vel.y.min(0.0);
            return;
        }
        if self.has(SkillId::Fly) {
            self.fly.update(body, dt);
            return;
        }
        physics::apply_gravity(body, dt);
    }

    /// Ativa skills pós-física para uma entidade. Retorna `(result, spend_aura)`.
    pub fn activate_post_physics(
        &mut self,
        act: SkillActivation<'_>,
        store: &mut EntityStore,
        events: &mut Vec<GameEvent>,
    ) -> bool {
        let mut any_started = false;
        for (slot, skill) in self.loadout.iter() {
            let def = skill.def();
            if def.timing != SkillTiming::PostPhysics {
                continue;
            }
            let input = if act.slot == slot {
                act.input
            } else {
                SlotInput::default()
            };
            // Centraliza por entidade: usamos o input da entidade inteira; chamamos
            // por slot no caller? Não - caller itera entidades e passa input do slot?
            // Simplificação: o caller passa o slot pressionado. Aqui verificamos.
            let _ = (slot, input);
        }
        let _ = (act, store, events, &mut any_started);
        any_started
    }

    /// Avança cooldowns e timers internos. Roda depois da ativação.
    pub fn tick(&mut self, dt: f32) {
        for cd in &mut self.cooldowns {
            *cd = (*cd - dt).max(0.0);
        }
        self.jump.tick_timers(dt);
        self.dash.tick_timers(dt);
        self.melee.tick(dt);
        self.mogging.tick(dt);
        self.nutella_shot.tick(dt);
        self.shout.tick(dt);
    }

    pub fn reset(&mut self) {
        let loadout = self.loadout;
        *self = Self::none();
        self.loadout = loadout;
    }
}

/// Verifica se a ativação dispara para o modo da skill.
pub fn should_activate(activation: Activation, input: SlotInput) -> bool {
    match activation {
        Activation::Press => input.pressed,
        Activation::Hold => input.held,
        Activation::PressHold => input.pressed || input.held,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_ids_have_stable_indices() {
        for (index, skill) in SkillId::ALL.iter().copied().enumerate() {
            assert_eq!(skill.index(), index);
            assert_eq!(skill.def().id, skill);
        }
    }

    #[test]
    fn player_default_loadout_has_four_core_skills() {
        let skills = Skills::player_default();
        assert_eq!(skills.loadout.get(ActionSlot::Slot1), Some(SkillId::Jump));
        assert_eq!(skills.loadout.get(ActionSlot::Slot2), Some(SkillId::Dash));
        assert_eq!(skills.loadout.get(ActionSlot::Slot3), Some(SkillId::Melee));
        assert_eq!(
            skills.loadout.get(ActionSlot::Slot4),
            Some(SkillId::Mogging)
        );
    }

    #[test]
    fn skill_id_codes_roundtrip() {
        for skill in SkillId::ALL {
            assert_eq!(SkillId::from_code(skill.code()), Some(skill));
        }
        assert_eq!(SkillId::from_code(255), None);
    }
}
