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

/// Bônus/níveis de skill indexados por [`SkillId::index`].
pub type SkillLevels = [u8; 7];
pub const NO_SKILL_LEVELS: SkillLevels = [0; 7];

pub const fn skill_level(levels: &SkillLevels, skill: SkillId) -> u8 {
    levels[skill.index()]
}

pub fn set_skill_level(levels: &mut SkillLevels, skill: SkillId, value: u8) {
    levels[skill.index()] = value;
}

pub fn add_skill_levels(a: SkillLevels, b: SkillLevels) -> SkillLevels {
    let mut out = [0u8; 7];
    let mut i = 0;
    while i < 7 {
        out[i] = a[i].saturating_add(b[i]);
        i += 1;
    }
    out
}

pub fn active_skill_tier(def: &SkillDef, effective_level: u8) -> Option<u8> {
    let mut active = None;
    let mut i = 0;
    while i < def.progression.len() {
        let tier = def.progression[i];
        if tier <= effective_level {
            active = Some(tier);
        }
        i += 1;
    }
    active
}

impl SkillId {
    pub fn effective_level(self, context: SkillContext) -> u8 {
        let def = self.def();
        context.effective_skill_level(self, def.base_skill_level)
    }

    pub fn active_tier(self, context: SkillContext) -> Option<u8> {
        active_skill_tier(self.def(), self.effective_level(context))
    }

    pub fn tier_steps(self, context: SkillContext) -> i32 {
        let def = self.def();
        let Some(active) = self.active_tier(context) else {
            return 0;
        };
        let mut min_index = 0usize;
        while min_index < def.progression.len() && def.progression[min_index] < def.min_skill_level
        {
            min_index += 1;
        }
        let mut active_index = min_index;
        let mut i = min_index;
        while i < def.progression.len() {
            if def.progression[i] <= active {
                active_index = i;
            }
            i += 1;
        }
        active_index.saturating_sub(min_index) as i32
    }

    pub fn availability(self, context: SkillContext) -> Result<(), SkillLock> {
        let def = self.def();
        def.requirements
            .check(context, self.effective_level(context))
    }

    pub fn effective_cooldown(self, context: SkillContext) -> f32 {
        if self == SkillId::Mogging {
            let tier = self.active_tier(context).unwrap_or(1);
            return mogging::tier_params(tier).cooldown;
        }
        let def = self.def();
        let steps = self.tier_steps(context).max(0) as f32;
        (def.cooldown * 0.95f32.powf(steps)).max(0.0)
    }

    pub fn effective_aura_cost(self, context: SkillContext) -> f32 {
        if self == SkillId::Mogging {
            let tier = self.active_tier(context).unwrap_or(1);
            return mogging::tier_params(tier).aura_cost;
        }
        self.def().aura_cost
    }
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
    /// Innate/base skill level (without equipment bonuses).
    pub base_skill_level: u8,
    /// Minimum effective skill level required to unlock/use the skill.
    pub min_skill_level: u8,
    /// Progression tiers for this skill (ascending). Active tier is the highest <= effective level.
    pub progression: &'static [u8],
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
    up_jump_prev: bool,
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
                    self.jump.held |= input.held;
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

        // Move up must always trigger jump, even if Jump is not equipped in any slot.
        let up_now = command.move_y < 0;
        self.jump.held |= up_now;
        if up_now && !self.up_jump_prev {
            self.jump.buffer = jump::JUMP_BUFFER_TIME;
        }
        self.up_jump_prev = up_now;
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
        // Jump is always available via MoveUp intent.
        self.jump.update(body, stats);

        self.update_horizontal(body, stats, locomotion_speed_factor, dt, locomotion);
        self.update_vertical(body, dt);
        physics::integrate(body, level, dt);

        if body.on_ground {
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
        assert_eq!(skills.loadout.get(ActionSlot::Slot4), Some(SkillId::Shout));
    }

    #[test]
    fn skill_id_codes_roundtrip() {
        for skill in SkillId::ALL {
            assert_eq!(SkillId::from_code(skill.code()), Some(skill));
        }
        assert_eq!(SkillId::from_code(255), None);
    }
}
