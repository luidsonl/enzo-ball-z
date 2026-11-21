//! Orquestração do mundo e passo fixo.

use crate::ai;
use crate::aura;
use crate::entities::{
    actors::enzo::ENZO_CROUCH_SIZE,
    actors::valentina::{VALENTINA_CROUCH_SIZE, VALENTINA_SIZE},
    item::ItemPickupMode,
    EntityId, EntitySpec, EntityStore, EquipError, EquipSlot, ItemData, ItemKind,
    PlayableCharacter, PlayerId, VisualProfile, ENZO_SIZE, SLIME_SIZE,
};
use crate::events::{GameEvent, SkillRejectReason, TickOutput};
use crate::input::{PlayerCommand, TickInput};
use crate::map::TILE_SIZE;
use crate::map::{BlockId, Level, LevelError};
use crate::math::Vec2;
use crate::mechanics::{self, Locomotion};
use crate::progression::{self, SaveData};
use crate::skills::{
    mogging, nutella_shot, punch, shout, LoadoutChange, LoadoutError, LoadoutRequest,
    SkillActivation, SkillId, SkillResult, SkillTiming,
};
use crate::snapshot::WorldSnapshot;
use crate::DT;

pub use crate::state::GamePhase;

/// Tempo entre a morte e o respawn automático.
pub const RESPAWN_DELAY: f32 = 1.2;

/// Duração da animação/estado de morte de inimigos antes do despawn.
pub const ENEMY_DEATH_DELAY: f32 = 0.4;

/// Distância abaixo do mapa em que a queda vira morte.
pub const FALL_DEATH_MARGIN: f32 = 64.0;

pub struct World {
    level: Level,
    initial_level: Level,
    store: EntityStore,
    tick: u64,
    phase: GamePhase,
    phase_timer: f32,
    pending_events: Vec<GameEvent>,
    sandbox_mode: bool,
}

impl World {
    pub fn new(source: &str) -> Result<Self, LevelError> {
        Self::new_with_mode(source, false)
    }

    pub fn new_sandbox(source: &str) -> Result<Self, LevelError> {
        Self::new_with_mode(source, true)
    }

    fn new_with_mode(source: &str, sandbox_mode: bool) -> Result<Self, LevelError> {
        let level = Level::parse(source)?;
        let mut store = EntityStore::new();

        let spawn = level
            .spawn_points(ENZO_SIZE)
            .first()
            .copied()
            .unwrap_or(Vec2::ZERO);
        store.spawn(EntitySpec::player(0, spawn));
        for spawn in &level.enemy_spawns {
            let pos = enemy_anchor(spawn.tile, &spawn.kind);
            spawn_enemy(&mut store, &spawn.kind, pos);
        }

        let initial_level = level.clone();
        Ok(Self {
            level,
            initial_level,
            store,
            tick: 0,
            phase: GamePhase::Playing,
            phase_timer: 0.0,
            pending_events: Vec::new(),
            sandbox_mode,
        })
    }

    pub fn is_sandbox(&self) -> bool {
        self.sandbox_mode
    }

    pub fn level(&self) -> &Level {
        &self.level
    }

    pub fn store(&self) -> &EntityStore {
        &self.store
    }

    pub fn store_mut(&mut self) -> &mut EntityStore {
        &mut self.store
    }

    pub fn phase(&self) -> GamePhase {
        self.phase
    }

    pub fn phase_timer(&self) -> f32 {
        self.phase_timer
    }

    pub fn tick(&self) -> u64 {
        self.tick
    }

    pub fn spawn_item(&mut self, item: ItemData, pos: Vec2) -> EntityId {
        self.store.spawn(EntitySpec::item(item, pos))
    }

    pub fn equip(&mut self, player_id: PlayerId, item: &ItemData) -> Result<(), EquipError> {
        let entity_id = self
            .store
            .player_entity_id(player_id)
            .ok_or(EquipError::NotEquippable)?;
        self.store.equip(entity_id, item)
    }

    /// Unequips a slot and drops the item into the world (inventory removed).
    pub fn unequip_slot(&mut self, player_id: PlayerId, slot: EquipSlot) -> bool {
        let (old, pos) = {
            let Some(entity) = self.store.player_mut(player_id) else {
                return false;
            };
            let Some(old) = entity.unequip(slot) else {
                return false;
            };
            (old, entity.body.pos)
        };
        self.store.spawn(EntitySpec::item(old.kind.data(), pos));
        self.pending_events.push(GameEvent::SaveRequested);
        true
    }

    /// Updates visual customization for a player (authoritative, persisted via save).
    pub fn set_visual_profile(&mut self, player_id: PlayerId, profile: VisualProfile) -> bool {
        let Some(entity) = self.store.player_mut(player_id) else {
            return false;
        };
        entity.visual_profile = profile;
        let Some(player) = entity.player_mut() else {
            return false;
        };
        player.visual_profile = profile;
        self.pending_events.push(GameEvent::SaveRequested);
        true
    }

    /// Updates which playable character a player controls. This updates identity/profile
    /// metadata; respawn/rebuild flows may be required by callers for full asset swaps.
    pub fn set_playable_character(
        &mut self,
        player_id: PlayerId,
        character: PlayableCharacter,
    ) -> bool {
        let Some(entity) = self.store.player_mut(player_id) else {
            return false;
        };
        entity.visual_character = Some(character);
        let Some(player) = entity.player_mut() else {
            return false;
        };
        player.character = character;
        self.pending_events.push(GameEvent::SaveRequested);
        true
    }

    pub fn save_data(&self) -> Option<SaveData> {
        self.store
            .player(0)
            .and_then(progression::SaveData::from_entity)
    }

    pub fn load_save(&mut self, save: &SaveData) -> bool {
        let Some(save) = save.migrate() else {
            return false;
        };
        self.store
            .player_mut(0)
            .is_some_and(|entity| save.apply_to(entity))
    }

    pub fn apply_loadout(
        &mut self,
        player_id: PlayerId,
        request: LoadoutRequest,
    ) -> Result<LoadoutChange, LoadoutError> {
        let Some(entity_id) = self.store.player_entity_id(player_id) else {
            return Err(LoadoutError::NoPlayer);
        };
        let Some(entity) = self.store.get_mut(entity_id) else {
            return Err(LoadoutError::NoPlayer);
        };
        if !entity.health.is_alive() {
            return Err(LoadoutError::NoPlayer);
        }
        let context = entity.skill_context();
        let change = match request {
            LoadoutRequest::Assign { slot, skill } => {
                if let Err(lock) = skill.availability(context) {
                    return Err(LoadoutError::Unavailable { skill, lock });
                }
                entity.skills.loadout.assign(slot, skill)
            }
            LoadoutRequest::Clear { slot } => match entity.skills.loadout.clear(slot) {
                Some(skill) => LoadoutChange::Cleared { slot, skill },
                None => return Err(LoadoutError::EmptySlot),
            },
        };
        self.pending_events.push(GameEvent::LoadoutChanged {
            player: entity_id,
            change,
        });
        self.pending_events.push(GameEvent::SaveRequested);
        Ok(change)
    }

    pub fn step(&mut self, input: TickInput) -> TickOutput {
        let mut events = std::mem::take(&mut self.pending_events);

        for index in 0..crate::MAX_PLAYERS {
            let Some(command) = input.command(index) else {
                continue;
            };
            let player_id = index as PlayerId;
            match self.phase {
                GamePhase::Playing => self.apply_playing_command(player_id, command, &mut events),
                GamePhase::LevelUp => self.apply_level_up_command(player_id, command, &mut events),
                GamePhase::Dead | GamePhase::Victory => {
                    if command.restart {
                        self.respawn(&mut events);
                    }
                }
            }
        }

        match self.phase {
            GamePhase::Playing => self.step_playing(&mut events),
            GamePhase::Dead | GamePhase::LevelUp | GamePhase::Victory => {}
        }

        self.clear_input_edges();
        self.tick = self.tick.wrapping_add(1);
        TickOutput::new(events)
    }

    fn apply_playing_command(
        &mut self,
        player_id: PlayerId,
        command: PlayerCommand,
        events: &mut Vec<GameEvent>,
    ) {
        let Some(entity_id) = self.store.player_entity_id(player_id) else {
            return;
        };

        if let Some(attribute) = command.allocate {
            if let Some(entity) = self.store.get_mut(entity_id) {
                let _ = progression::allocate_point(entity, attribute, events);
            }
        }

        if command.confirm {
            if let Some(entity) = self.store.get_mut(entity_id) {
                progression::confirm_level_up(entity);
            }
        }

        let Some(entity) = self.store.player_mut(player_id) else {
            return;
        };
        if !entity.health.is_alive() {
            return;
        }
        if let Some(controller) = entity.controller.player_controller_mut() {
            controller.apply(command);
        }
        entity.locomotion.apply_command(command);
        // intenção de slots pré-física será lida do controller/input no step.
    }

    fn apply_level_up_command(
        &mut self,
        player_id: PlayerId,
        command: PlayerCommand,
        events: &mut Vec<GameEvent>,
    ) {
        let Some(entity_id) = self.store.player_entity_id(player_id) else {
            return;
        };

        if let Some(attribute) = command.allocate {
            if let Some(entity) = self.store.get_mut(entity_id) {
                let _ = progression::allocate_point(entity, attribute, events);
            }
        }

        if command.confirm {
            if let Some(entity) = self.store.get_mut(entity_id) {
                progression::confirm_level_up(entity);
            }
            self.phase = GamePhase::Playing;
        }
    }

    fn step_playing(&mut self, events: &mut Vec<GameEvent>) {
        // 1. SixSeven
        for entity in self.store.players_mut() {
            let max_aura = entity.stats.max_aura;
            let key = entity.locomotion.take_sixseven_request();
            if let Some(key) = key {
                let sixseven = &mut entity.locomotion.sixseven;
                if let Some(player) = entity.kind.player_mut() {
                    aura::apply_sixseven(player, sixseven, key, max_aura);
                }
            }
        }

        // 2-4. Intent pré-física + física
        let level = &self.level;
        let fall_limit = level.height_px() + FALL_DEATH_MARGIN;
        for entity in self.store.iter_mut() {
            if !entity.is_actor() || !entity.health.is_alive() {
                continue;
            }
            // Pré-física: intenção de walk já está no locomotion; skills leem controller.
            let command = entity
                .controller
                .player_controller()
                .map(|c| PlayerCommand {
                    move_x: c.input.move_x,
                    move_y: c.input.move_y,
                    slots: c.input.slots,
                    ..PlayerCommand::none()
                })
                .unwrap_or_else(PlayerCommand::none);
            let crouch_sizes = crouch_sizes_for(entity);
            if let Some((stand, crouch)) = crouch_sizes {
                apply_crouch_posture(entity, level, stand, crouch);
            }
            entity.skills.apply_intent(command, &entity.body);
            let mut speed_factor = entity.locomotion.speed_factor();
            if entity.locomotion.crouching && entity.body.on_ground {
                speed_factor *= 0.5;
            }
            let mut locomotion = entity.locomotion;
            entity.skills.step_physics(
                &mut entity.body,
                &entity.stats,
                level,
                speed_factor,
                DT,
                &mut locomotion,
            );
            entity.locomotion = locomotion;

            if entity.is_actor() && entity.body.pos.y > fall_limit {
                // Falling below the map is lethal even in Sigma state.
                entity.health.current = 0.0;
            }
        }

        // 2b. World item physics (modular per item type)
        for entity in self.store.iter_mut() {
            if !entity.is_item() {
                continue;
            }
            let Some(physics_cfg) = entity.item().map(|item| item.world_physics()) else {
                continue;
            };
            if !physics_cfg.gravity && !physics_cfg.collides_level {
                continue;
            }
            if physics_cfg.gravity {
                crate::physics::apply_gravity(&mut entity.body, DT);
            }
            if physics_cfg.collides_level {
                crate::physics::integrate(&mut entity.body, level, DT);
            } else {
                entity.body.pos += entity.body.vel * DT;
                entity.body.on_ground = false;
            }
        }

        // Despawn world items that fell far below the level.
        let fallen_items: Vec<EntityId> = self
            .store
            .iter()
            .filter(|e| e.is_item() && e.body.pos.y > fall_limit)
            .map(|e| e.id)
            .collect();
        for id in fallen_items {
            self.store.despawn(id);
        }

        // 5. Skills pós-física
        for index in 0..crate::MAX_PLAYERS {
            let player_id = index as PlayerId;
            let Some(entity_id) = self.store.player_entity_id(player_id) else {
                continue;
            };
            self.activate_post_physics(entity_id, events);
        }

        // 6. Projectiles (may damage destructible blocks)
        mogging::update_projectiles(&mut self.store, &mut self.level, DT, events);

        // 7. AI
        ai::update(&mut self.store, &self.level);

        // 8. Colisões
        mechanics::combat::contact_damage(&mut self.store, events);
        mechanics::combat::tick_timers(&mut self.store, DT);

        // 9. Timers de skills
        for entity in self.store.iter_mut() {
            entity.skills.tick(DT);
            entity.locomotion.tick(DT);
        }

        // 9b. World item pickup/collection (modular per item type).
        self.collect_world_items(events);

        // Sigma state: aura recovers to 100% in 3 seconds while active.
        for entity in self.store.players_mut() {
            if !entity.health.is_alive() {
                continue;
            }
            let max_aura = entity.stats.max_aura;
            let Some(player) = entity.player_mut() else {
                continue;
            };
            if !player.sigma.is_active() {
                continue;
            }
            let recover = max_aura / 3.0 * DT;
            player.aura = (player.aura + recover).min(max_aura);
        }

        // 10. Progressão
        progression::process_kills(&mut self.store, events);

        // 11. Limpeza
        let mut dead_to_despawn = Vec::new();
        for entity in self.store.enemies_mut() {
            if entity.health.is_alive() {
                continue;
            }
            let Some(enemy) = entity.enemy_mut() else {
                continue;
            };
            if enemy.death_timer <= 0.0 {
                enemy.death_timer = ENEMY_DEATH_DELAY;
            } else {
                enemy.death_timer = (enemy.death_timer - DT).max(0.0);
            }
            if enemy.death_timer <= 0.0 {
                dead_to_despawn.push(entity.id);
            }
        }
        for id in dead_to_despawn {
            self.store.despawn(id);
        }

        let dead_players: Vec<EntityId> = self
            .store
            .players()
            .filter(|entity| !entity.health.is_alive())
            .map(|entity| entity.id)
            .collect();
        if !dead_players.is_empty() {
            for player in &dead_players {
                events.push(GameEvent::PlayerDied { player: *player });
                self.apply_death_progression(*player, events);
            }
            if self.sandbox_mode {
                self.sandbox_handle_player_deaths(events);
                return;
            }
            self.phase = GamePhase::Dead;
            self.phase_timer = RESPAWN_DELAY;
            return;
        }

        if let Some(flag) = self.level.flag_rect() {
            let reached = self
                .store
                .players()
                .any(|entity| entity.health.is_alive() && entity.rect().overlaps(flag));
            if reached {
                if let Some(player) = self.store.players().next().map(|entity| entity.id) {
                    events.push(GameEvent::Victory { player });
                }
                events.push(GameEvent::SaveRequested);
                self.phase = GamePhase::Victory;
            }
        }
    }

    fn activate_post_physics(&mut self, entity_id: EntityId, events: &mut Vec<GameEvent>) {
        let Some(snapshot) = self.store.get(entity_id) else {
            return;
        };
        if !snapshot.health.is_alive() || !snapshot.is_player() {
            return;
        }
        let Some(controller) = snapshot.controller.player_controller().copied() else {
            return;
        };
        let context = snapshot.skill_context();
        let loadout = snapshot.skills.loadout;
        let cooldowns = snapshot.skills.cooldowns;
        let stats = snapshot.stats;
        let rect = snapshot.rect();
        let facing = snapshot.body.facing;
        let team = snapshot.team;
        let player_aura = snapshot.aura();

        for (slot, skill) in loadout.iter() {
            let def = skill.def();
            if def.timing != SkillTiming::PostPhysics {
                continue;
            }
            let input = controller.input.slots[slot.index()];
            let wants = crate::skills::should_activate(def.activation, input);
            if !wants {
                continue;
            }
            if let Err(lock) = skill.availability(context) {
                events.push(GameEvent::SkillRejected {
                    player: entity_id,
                    slot,
                    skill,
                    reason: SkillRejectReason::Unavailable(lock),
                });
                continue;
            }
            let aura_cost = skill.effective_aura_cost(context);
            let cooldown = skill.effective_cooldown(context);
            if cooldowns[skill.index()] > 0.0 {
                events.push(GameEvent::SkillRejected {
                    player: entity_id,
                    slot,
                    skill,
                    reason: SkillRejectReason::Cooldown,
                });
                continue;
            }
            if player_aura < aura_cost {
                events.push(GameEvent::SkillRejected {
                    player: entity_id,
                    slot,
                    skill,
                    reason: SkillRejectReason::NoAura,
                });
                continue;
            }

            let act = SkillActivation {
                entity_id,
                slot,
                input,
                move_x: controller.input.move_x,
                move_y: controller.input.move_y,
                rect,
                facing,
                aura: player_aura,
                stats: &stats,
                team,
                tick: self.tick,
            };

            let result = match skill {
                SkillId::Melee => {
                    let Some(entity) = self.store.get_mut(entity_id) else {
                        return;
                    };
                    let mut state = entity.skills.punch;
                    let r = punch::activate(&mut state, act, &mut self.store, events);
                    if let Some(entity) = self.store.get_mut(entity_id) {
                        entity.skills.punch = state;
                    }
                    r
                }
                SkillId::Mogging => {
                    let Some(entity) = self.store.get_mut(entity_id) else {
                        return;
                    };
                    let mut state = entity.skills.mogging;
                    let r = mogging::activate(&mut state, act, &mut self.store, events);
                    if let Some(entity) = self.store.get_mut(entity_id) {
                        entity.skills.mogging = state;
                    }
                    r
                }
                SkillId::NutellaShot => {
                    let Some(entity) = self.store.get_mut(entity_id) else {
                        return;
                    };
                    let mut state = entity.skills.nutella_shot;
                    let r = nutella_shot::activate(&mut state, act, &mut self.store, events);
                    if let Some(entity) = self.store.get_mut(entity_id) {
                        entity.skills.nutella_shot = state;
                    }
                    r
                }
                SkillId::Shout => {
                    let Some(entity) = self.store.get_mut(entity_id) else {
                        return;
                    };
                    let mut state = entity.skills.shout;
                    let r = shout::activate(&mut state, act, &mut self.store, events);
                    if let Some(entity) = self.store.get_mut(entity_id) {
                        entity.skills.shout = state;
                    }
                    r
                }
                _ => SkillResult::Idle,
            };

            if matches!(result, SkillResult::Started) {
                if let Some(entity) = self.store.get_mut(entity_id) {
                    if let Some(player) = entity.player_mut() {
                        if aura_cost > 0.0 {
                            let _ = aura::spend(player, aura_cost);
                        }
                    }
                    if cooldown > 0.0 {
                        entity.skills.cooldowns[skill.index()] = cooldown + DT;
                    }
                }
            }
        }
    }

    fn collect_world_items(&mut self, events: &mut Vec<GameEvent>) {
        #[derive(Clone, Copy)]
        struct PickupDecision {
            player_eid: EntityId,
            item_eid: EntityId,
        }
        #[derive(Clone, Copy)]
        struct Candidate {
            item_eid: EntityId,
            mode: ItemPickupMode,
            priority: i32,
            dist2: i64,
        }

        let player_ids: Vec<EntityId> = self.store.players().map(|e| e.id).collect();
        let mut consumed_items: std::collections::BTreeSet<u32> = std::collections::BTreeSet::new();
        let mut decisions: Vec<PickupDecision> = Vec::new();

        for player_eid in &player_ids {
            let Some(player) = self.store.get(*player_eid) else {
                continue;
            };
            if !player.health.is_alive() || player.player().is_none() {
                continue;
            }
            let interact_pressed = player
                .controller
                .player_controller()
                .is_some_and(|c| c.input.interact);
            let player_rect = player.rect();
            let pc = player_rect.center();

            let mut candidates: Vec<Candidate> = self
                .store
                .iter()
                .filter_map(|e| {
                    if !e.is_item() {
                        return None;
                    }
                    let item = e.item().copied()?;
                    let rect = e.rect();
                    if !player_rect.overlaps(rect) {
                        return None;
                    }
                    let ic = rect.center();
                    let dx = (ic.x - pc.x) as i64;
                    let dy = (ic.y - pc.y) as i64;
                    let pickup = item.pickup();
                    Some(Candidate {
                        item_eid: e.id,
                        mode: pickup.mode,
                        priority: pickup.priority,
                        dist2: dx * dx + dy * dy,
                    })
                })
                .collect();

            // Deterministic base order (priority for interact is applied below).
            candidates.sort_unstable_by_key(|c| (c.dist2, c.item_eid.0));

            // Touch/instant-use items: auto-collect on overlap.
            for c in &candidates {
                if consumed_items.contains(&c.item_eid.0) {
                    continue;
                }
                if !matches!(c.mode, ItemPickupMode::Touch) {
                    continue;
                }
                decisions.push(PickupDecision {
                    player_eid: *player_eid,
                    item_eid: c.item_eid,
                });
                consumed_items.insert(c.item_eid.0);
            }

            // Interact items: action button picks one by collision/pickup priority.
            if interact_pressed {
                let mut interact_candidates: Vec<Candidate> = candidates
                    .iter()
                    .copied()
                    .filter(|c| {
                        matches!(c.mode, ItemPickupMode::Interact)
                            && !consumed_items.contains(&c.item_eid.0)
                    })
                    .collect();
                interact_candidates.sort_unstable_by(|a, b| {
                    b.priority
                        .cmp(&a.priority)
                        .then_with(|| a.dist2.cmp(&b.dist2))
                        .then_with(|| a.item_eid.0.cmp(&b.item_eid.0))
                });
                if let Some(best) = interact_candidates.first() {
                    decisions.push(PickupDecision {
                        player_eid: *player_eid,
                        item_eid: best.item_eid,
                    });
                    consumed_items.insert(best.item_eid.0);
                }
            }
        }

        for decision in decisions {
            self.apply_item_pickup(decision.player_eid, decision.item_eid, events);
        }
    }

    fn apply_item_pickup(
        &mut self,
        player_eid: EntityId,
        item_eid: EntityId,
        _events: &mut Vec<GameEvent>,
    ) {
        let Some((item_data, drop_pos)) = self
            .store
            .get(item_eid)
            .and_then(|e| e.item().copied().map(|data| (data, e.body.pos)))
        else {
            return;
        };
        let Some(player_snapshot) = self.store.get(player_eid) else {
            return;
        };
        if !player_snapshot.health.is_alive() || player_snapshot.player().is_none() {
            return;
        }

        // General usability/requirements gate (entity type + character attributes).
        if item_data
            .requirements
            .allows(
                player_snapshot.entity_type(),
                player_snapshot.character_attribute_set(),
            )
            .is_err()
        {
            return;
        }

        if item_data.is_consumable() {
            // Consume immediately (no inventory).
            if let Some(player) = self.store.get_mut(player_eid) {
                if item_data.heal > 0.0 {
                    let max = player.stats.max_health;
                    player.health.current = (player.health.current + item_data.heal).min(max);
                }
            }
            self.store.despawn(item_eid);
            self.pending_events.push(GameEvent::SaveRequested);
            return;
        }

        // Equippables: place directly into the character slot.
        let Some(slot) = item_data.slot() else {
            return;
        };
        // Validate equipment-specific eligibility (same as direct equip).
        {
            let Some(player) = self.store.get(player_eid) else {
                return;
            };
            if item_data
                .can_be_equipped_by(player.entity_type(), player.character_attribute_set())
                .is_err()
            {
                return;
            }
        }

        let drop_old = {
            let Some(player) = self.store.get_mut(player_eid) else {
                return;
            };
            let old = player.unequip(slot);
            if player.try_equip(&item_data).is_err() {
                // Restore previous item if equip unexpectedly fails.
                if let Some(old_item) = old {
                    let _ = player.try_equip(&old_item.kind.data());
                }
                return;
            }
            old
        };

        self.store.despawn(item_eid);
        if let Some(old_item) = drop_old {
            let drop_at = self
                .store
                .get(player_eid)
                .map(|p| p.body.pos)
                .unwrap_or(drop_pos);
            self.store
                .spawn(EntitySpec::item(old_item.kind.data(), drop_at));
        }
        self.pending_events.push(GameEvent::SaveRequested);
    }

    fn clear_input_edges(&mut self) {
        for entity in self.store.players_mut() {
            if let Some(controller) = entity.controller.player_controller_mut() {
                controller.clear_edges();
            }
        }
    }

    fn respawn(&mut self, events: &mut Vec<GameEvent>) {
        let spawn = self
            .level
            .spawn_points(ENZO_SIZE)
            .first()
            .copied()
            .unwrap_or(Vec2::ZERO);

        for entity in self.store.players_mut() {
            entity.body.pos = spawn;
            entity.body.vel = Vec2::ZERO;
            entity.body.on_ground = false;
            entity.health.current = entity.stats.max_health;
            entity.health.iframes = 0.0;
            let max_aura = entity.stats.max_aura;
            if let Some(player) = entity.player_mut() {
                player.aura = max_aura;
                player.respawn = spawn;
                player.pending_level_up = player.points > 0;
            }
            entity.locomotion = Locomotion::player();
            entity.skills.reset();
            if let Some(controller) = entity.controller.player_controller_mut() {
                *controller = Default::default();
            }
        }

        let removable: Vec<EntityId> = self
            .store
            .iter()
            .filter(|entity| entity.is_enemy() || entity.is_projectile() || entity.is_item())
            .map(|entity| entity.id)
            .collect();
        for id in removable {
            self.store.despawn(id);
        }
        for spawn in &self.level.enemy_spawns {
            let pos = enemy_anchor(spawn.tile, &spawn.kind);
            spawn_enemy(&mut self.store, &spawn.kind, pos);
        }

        self.phase = GamePhase::Playing;
        self.phase_timer = 0.0;
        events.push(GameEvent::PhaseRestarted { player: 0 });
    }

    fn apply_death_progression(&mut self, eid: EntityId, events: &mut Vec<GameEvent>) {
        let Some(entity) = self.store.get_mut(eid) else {
            return;
        };
        if entity.player().is_none() {
            return;
        }
        let mut rng =
            crate::rng::Rng::new(self.tick ^ (eid.0 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let _ = progression::apply_death_level_loss(entity, &mut rng, events);
    }

    fn sandbox_handle_player_deaths(&mut self, _events: &mut Vec<GameEvent>) {
        let player_entity_ids: Vec<EntityId> = self.store.players().map(|e| e.id).collect();
        for eid in player_entity_ids {
            let dead_or_should_handle = self
                .store
                .get(eid)
                .is_some_and(|e| e.player().is_some() && !e.health.is_alive());
            if !dead_or_should_handle {
                continue;
            }
            // Progression penalty is applied on death detection; sandbox respawns immediately.
            self.sandbox_respawn_player(eid);
        }
        self.phase = GamePhase::Playing;
        self.phase_timer = 0.0;
    }

    fn sandbox_respawn_player(&mut self, eid: EntityId) {
        let spawn = self
            .level
            .spawn_points(ENZO_SIZE)
            .first()
            .copied()
            .unwrap_or(Vec2::ZERO);
        let Some(entity) = self.store.get_mut(eid) else {
            return;
        };
        if entity.player().is_none() {
            return;
        }
        entity.body.pos = spawn;
        entity.body.vel = Vec2::ZERO;
        entity.body.on_ground = false;
        entity.health.iframes = 0.0;
        let max_aura = entity.stats.max_aura;
        let max_health = entity.stats.max_health;
        entity.health.current = max_health.max(1.0);
        if let Some(player) = entity.player_mut() {
            player.aura = max_aura;
            player.respawn = spawn;
            player.pending_level_up = player.points > 0;
        }
        entity.locomotion = Locomotion::player();
        entity.skills.reset();
        if let Some(controller) = entity.controller.player_controller_mut() {
            *controller = Default::default();
        }
    }

    // --- Sandbox/debug APIs (authoritative core mutations) ---

    pub fn sandbox_set_sigma_mode(&mut self, player_id: PlayerId, enabled: bool) -> bool {
        if !self.sandbox_mode {
            return false;
        }
        let Some(entity) = self.store.player_mut(player_id) else {
            return false;
        };
        let max_health = entity.stats.max_health;
        {
            let Some(player) = entity.player_mut() else {
                return false;
            };
            player.sigma.set_active(enabled);
        }
        if enabled {
            // Keep player healthy when entering Sigma state; aura recovers over time while active.
            entity.health.current = entity.health.current.max(max_health);
        }
        true
    }

    pub fn sandbox_debug_set_level(&mut self, player_id: PlayerId, level: u32) -> bool {
        if !self.sandbox_mode {
            return false;
        }
        let Some(entity) = self.store.player_mut(player_id) else {
            return false;
        };
        progression::debug_set_level(entity, level)
    }

    pub fn sandbox_debug_grant_levels(&mut self, player_id: PlayerId, levels: u32) -> bool {
        if !self.sandbox_mode {
            return false;
        }
        let Some(entity) = self.store.player_mut(player_id) else {
            return false;
        };
        progression::debug_grant_levels(entity, levels)
    }

    pub fn sandbox_debug_reset_attributes(&mut self, player_id: PlayerId) -> bool {
        if !self.sandbox_mode {
            return false;
        }
        let Some(entity) = self.store.player_mut(player_id) else {
            return false;
        };
        progression::debug_reset_attributes(entity)
    }

    pub fn sandbox_debug_die(&mut self, player_id: PlayerId) -> bool {
        if !self.sandbox_mode {
            return false;
        }
        let eid = match self.store.player_entity_id(player_id) {
            Some(id) => id,
            None => return false,
        };
        // Debug "Die" is allowed even while Sigma state is active.
        let mut events = Vec::new();
        self.apply_death_progression(eid, &mut events);
        self.sandbox_respawn_player(eid);
        self.phase = GamePhase::Playing;
        self.phase_timer = 0.0;
        true
    }

    pub fn sandbox_reset(&mut self) -> bool {
        if !self.sandbox_mode {
            return false;
        }
        self.level = self.initial_level.clone();
        let spawn = self
            .level
            .spawn_points(ENZO_SIZE)
            .first()
            .copied()
            .unwrap_or(Vec2::ZERO);

        // Preserve character/profile customization and Sigma state. Reset to default
        // sandbox player state while keeping selected character/profile and state.
        let (character, profile, sigma_state) = self
            .store
            .player(0)
            .and_then(|e| e.player().map(|p| (p.character, p.visual_profile, p.sigma)))
            .unwrap_or((
                PlayableCharacter::Enzo,
                VisualProfile::default(),
                crate::entities::SigmaState::Off,
            ));

        self.store = EntityStore::new();
        let mut spec = EntitySpec::player(0, spawn);
        spec.visual_character = Some(character);
        spec.visual_profile = profile;
        let pid = self.store.spawn(spec);
        if let Some(entity) = self.store.get_mut(pid) {
            if let Some(player) = entity.player_mut() {
                player.character = character;
                player.visual_profile = profile;
                player.sigma = sigma_state;
                player.respawn = spawn;
            }
        }

        for spawn_def in &self.level.enemy_spawns {
            let pos = enemy_anchor(spawn_def.tile, &spawn_def.kind);
            spawn_enemy(&mut self.store, &spawn_def.kind, pos);
        }

        self.tick = 0;
        self.phase = GamePhase::Playing;
        self.phase_timer = 0.0;
        self.pending_events.clear();
        true
    }

    pub fn sandbox_set_block(&mut self, tx: i32, ty: i32, block: BlockId) -> bool {
        if !self.sandbox_mode || tx < 0 || ty < 0 {
            return false;
        }
        self.level.set_block(tx as usize, ty as usize, block)
    }

    pub fn sandbox_remove_block(&mut self, tx: i32, ty: i32) -> bool {
        if !self.sandbox_mode || tx < 0 || ty < 0 {
            return false;
        }
        self.level.remove_block(tx as usize, ty as usize)
    }

    pub fn sandbox_spawn_enemy(&mut self, kind: &str, pos: Vec2) -> Option<EntityId> {
        if !self.sandbox_mode {
            return None;
        }
        if !(0.0..self.level.width_px()).contains(&pos.x)
            || !(0.0..self.level.height_px()).contains(&pos.y)
        {
            return None;
        }
        let id = match kind {
            "slime" | "" => self.store.spawn(EntitySpec::slime(pos)),
            _ => return None,
        };
        Some(id)
    }

    pub fn sandbox_spawn_item(&mut self, kind: ItemKind, pos: Vec2) -> Option<EntityId> {
        if !self.sandbox_mode {
            return None;
        }
        if !(0.0..self.level.width_px()).contains(&pos.x)
            || !(0.0..self.level.height_px()).contains(&pos.y)
        {
            return None;
        }
        Some(self.store.spawn(EntitySpec::item(kind.data(), pos)))
    }

    pub fn sandbox_clear_dynamic_entities(&mut self) -> bool {
        if !self.sandbox_mode {
            return false;
        }
        let removable: Vec<EntityId> = self
            .store
            .iter()
            .filter(|entity| entity.is_enemy() || entity.is_projectile() || entity.is_item())
            .map(|entity| entity.id)
            .collect();
        for id in removable {
            self.store.despawn(id);
        }
        true
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        let mut snapshot = WorldSnapshot {
            version: crate::snapshot::SNAPSHOT_VERSION,
            tick: self.tick,
            phase: self.phase,
            phase_timer: self.phase_timer,
            entities: Vec::new(),
            level_width: self.level.width_px(),
            level_height: self.level.height_px(),
            flag: self.level.flag_rect(),
            camera_hint: self.camera_hint(),
        };
        let tick = self.tick;
        snapshot.entities.extend(
            self.store
                .iter()
                .map(|e| crate::snapshot::EntitySnapshot::from_entity_at_tick(e, tick)),
        );
        snapshot
    }

    pub fn write_snapshot(&self, out: &mut WorldSnapshot) {
        out.version = crate::snapshot::SNAPSHOT_VERSION;
        out.tick = self.tick;
        out.phase = self.phase;
        out.phase_timer = self.phase_timer;
        out.level_width = self.level.width_px();
        out.level_height = self.level.height_px();
        out.flag = self.level.flag_rect();
        out.camera_hint = self.camera_hint();
        out.entities.clear();
        let tick = self.tick;
        out.entities.extend(
            self.store
                .iter()
                .map(|e| crate::snapshot::EntitySnapshot::from_entity_at_tick(e, tick)),
        );
    }

    fn camera_hint(&self) -> Vec2 {
        self.store
            .player(0)
            .map(|entity| entity.body.pos)
            .unwrap_or(Vec2::ZERO)
    }
}

fn enemy_anchor(tile: (usize, usize), kind: &str) -> Vec2 {
    let size = match kind {
        "slime" => SLIME_SIZE,
        _ => SLIME_SIZE,
    };
    Vec2::new(
        tile.0 as f32 * TILE_SIZE + (TILE_SIZE - size.x) * 0.5,
        (tile.1 as f32 + 1.0) * TILE_SIZE - size.y,
    )
}

fn spawn_enemy(store: &mut EntityStore, kind: &str, pos: Vec2) {
    match kind {
        "slime" | "" => {
            store.spawn(EntitySpec::slime(pos));
        }
        _ => {
            store.spawn(EntitySpec::slime(pos));
        }
    }
}

fn crouch_sizes_for(entity: &crate::entities::Entity) -> Option<(Vec2, Vec2)> {
    if !entity.is_player() {
        return None;
    }
    match entity.visual_character.unwrap_or(PlayableCharacter::Enzo) {
        PlayableCharacter::Enzo => Some((ENZO_SIZE, ENZO_CROUCH_SIZE)),
        PlayableCharacter::Valentina => Some((VALENTINA_SIZE, VALENTINA_CROUCH_SIZE)),
    }
}

fn apply_crouch_posture(
    entity: &mut crate::entities::Entity,
    level: &Level,
    stand: Vec2,
    crouch: Vec2,
) {
    let wants_crouch = entity.locomotion.crouch_intent && entity.body.on_ground;
    if wants_crouch {
        set_body_height_keep_feet(&mut entity.body, crouch);
        entity.locomotion.crouching = true;
        return;
    }

    if entity.locomotion.crouching || (entity.body.size.y - crouch.y).abs() < 0.01 {
        if can_stand_up(&entity.body, stand, level) {
            set_body_height_keep_feet(&mut entity.body, stand);
            entity.locomotion.crouching = false;
        } else {
            set_body_height_keep_feet(&mut entity.body, crouch);
            entity.locomotion.crouching = true;
        }
    } else if (entity.body.size.y - stand.y).abs() > 0.01
        && can_stand_up(&entity.body, stand, level)
    {
        set_body_height_keep_feet(&mut entity.body, stand);
    }
}

fn set_body_height_keep_feet(body: &mut crate::entities::Body, target: Vec2) {
    let bottom = body.bottom();
    body.size = target;
    body.pos.y = bottom - target.y;
}

fn can_stand_up(body: &crate::entities::Body, stand: Vec2, level: &Level) -> bool {
    let bottom = body.bottom();
    let top = bottom - stand.y;
    let rect = crate::math::Rect::new(body.pos.x, top, stand.x, stand.y);
    !level.rect_overlaps_solid(rect)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{EquipSlot, ItemKind};
    use crate::input::ActionSlot;
    use crate::levels::LEVEL_1;
    use crate::skills::{LoadoutChange, LoadoutRequest, SkillId};

    #[test]
    fn apply_loadout_swaps_skills() {
        let mut world = World::new(LEVEL_1).unwrap();
        // Mogging requires skill level 1 (granted by 67 items).
        world.equip(0, &ItemKind::Cap67.data()).unwrap();
        let change = world
            .apply_loadout(
                0,
                LoadoutRequest::Assign {
                    slot: ActionSlot::Slot4,
                    skill: SkillId::Mogging,
                },
            )
            .unwrap();
        assert!(matches!(change, LoadoutChange::Assigned { .. }));
        let entity = world.store().player(0).unwrap();
        assert_eq!(
            entity.skills.loadout.slot_of(SkillId::Mogging),
            Some(ActionSlot::Slot4)
        );
        assert!(!entity.skills.loadout.contains(SkillId::Shout));

        // Swapping two equipped available skills still works.
        let change = world
            .apply_loadout(
                0,
                LoadoutRequest::Assign {
                    slot: ActionSlot::Slot1,
                    skill: SkillId::Mogging,
                },
            )
            .unwrap();
        assert!(matches!(change, LoadoutChange::Swapped { .. }));
        let entity = world.store().player(0).unwrap();
        assert_eq!(
            entity.skills.loadout.slot_of(SkillId::Mogging),
            Some(ActionSlot::Slot1)
        );
        assert_eq!(
            entity.skills.loadout.slot_of(SkillId::Jump),
            Some(ActionSlot::Slot4)
        );
    }

    #[test]
    fn direct_equip_replaces_same_slot() {
        let mut world = World::new(LEVEL_1).unwrap();
        world.equip(0, &ItemKind::Cap67.data()).unwrap();
        assert!(world.store().player(0).unwrap().equipped[EquipSlot::Head.index()].is_some());
        world.equip(0, &ItemKind::Shirt67.data()).unwrap();
        assert!(world.store().player(0).unwrap().equipped[EquipSlot::Armor.index()].is_some());
    }

    #[test]
    fn unequip_slot_drops_item_into_world() {
        let mut world = World::new(LEVEL_1).unwrap();
        world.equip(0, &ItemKind::Sneakers67.data()).unwrap();
        let items_before = world.store().items().count();
        assert!(world.unequip_slot(0, EquipSlot::Feet));
        let entity = world.store().player(0).unwrap();
        assert!(entity.equipped[EquipSlot::Feet.index()].is_none());
        assert_eq!(world.store().items().count(), items_before + 1);
    }

    #[test]
    fn set_visual_profile_updates_player_customization() {
        let mut world = World::new(LEVEL_1).unwrap();
        let profile = VisualProfile {
            skin_tone: 2,
            hair_style: 3,
            hair_color: 1,
            face: 2,
            outfit_top: 1,
            outfit_top_detail1: 4,
            outfit_top_detail2: 6,
            outfit_bottom: 1,
            shoes: 2,
        };
        assert!(world.set_visual_profile(0, profile));
        let entity = world.store().player(0).unwrap();
        assert_eq!(entity.player().unwrap().visual_profile, profile);
        assert_eq!(entity.player().unwrap().character, PlayableCharacter::Enzo);
    }

    #[test]
    fn sandbox_sigma_can_die_by_falling() {
        let mut world = World::new_sandbox(crate::levels::SANDBOX_LEVEL).unwrap();
        assert!(world.sandbox_set_sigma_mode(0, true));
        assert!(world.sandbox_debug_set_level(0, 3));
        let below = world.level().height_px() + FALL_DEATH_MARGIN + 32.0;
        if let Some(player) = world.store_mut().player_mut(0) {
            player.body.pos.y = below;
            player.body.vel.y = 500.0;
        }
        for _ in 0..10 {
            world.step(crate::input::TickInput::default());
        }
        let player = world.store().player(0).unwrap();
        let pdata = player.player().unwrap();
        assert!(player.health.is_alive()); // respawned in sandbox
        assert!(pdata.sigma.is_active());
        assert_eq!(pdata.level, 2);
        assert_ne!(world.phase(), GamePhase::Dead);
    }

    #[test]
    fn sandbox_death_reduces_unspent_first_and_takes_deficit_from_applied() {
        let mut world = World::new_sandbox(crate::levels::SANDBOX_LEVEL).unwrap();
        assert!(world.sandbox_debug_set_level(0, 3)); // 10 points
        if let Some(entity) = world.store_mut().player_mut(0) {
            // 7 applied, 3 unspent.
            let mut events = Vec::new();
            for attr in [
                crate::input::Attribute::Speed,
                crate::input::Attribute::Speed,
                crate::input::Attribute::Strength,
                crate::input::Attribute::Strength,
                crate::input::Attribute::Health,
                crate::input::Attribute::Health,
                crate::input::Attribute::Aura,
            ] {
                progression::allocate_point(entity, attr, &mut events);
            }
        }
        assert!(world.sandbox_debug_die(0));
        let player = world.store().player(0).unwrap();
        let pdata = player.player().unwrap();
        assert_eq!(pdata.level, 2);
        // 5 points lost: 3 unspent consumed, 2 removed from applied status.
        assert_eq!(pdata.points, 0);
        assert_eq!(pdata.xp, 0.0);
        let total = player.attributes.speed
            + player.attributes.strength
            + player.attributes.health
            + player.attributes.aura
            + player.attributes.ego;
        assert_eq!(total, 5);
        assert!(player.health.is_alive());
        assert_ne!(world.phase(), GamePhase::Dead);
    }

    #[test]
    fn sandbox_block_edit_and_spawns_work() {
        let mut world = World::new_sandbox(crate::levels::SANDBOX_LEVEL).unwrap();
        assert!(world.sandbox_set_block(5, 5, crate::map::BlockId::Block));
        assert_eq!(world.level().tile(5, 5), crate::map::Tile::BLOCK);
        assert!(world.sandbox_remove_block(5, 5));
        assert_eq!(world.level().tile(5, 5), crate::map::Tile::EMPTY);

        let enemy_count_before = world.store().enemies().count();
        assert!(world
            .sandbox_spawn_enemy("slime", crate::math::Vec2::new(200.0, 100.0))
            .is_some());
        assert_eq!(world.store().enemies().count(), enemy_count_before + 1);

        let item_count_before = world.store().items().count();
        assert!(world
            .sandbox_spawn_item(ItemKind::Sword, crate::math::Vec2::new(240.0, 100.0))
            .is_some());
        assert_eq!(world.store().items().count(), item_count_before + 1);
    }

    #[test]
    fn item_gravity_is_modular_per_item_type() {
        let mut world = World::new_sandbox(crate::levels::SANDBOX_LEVEL).unwrap();
        let pos = crate::math::Vec2::new(240.0, 120.0);
        let falling = world
            .sandbox_spawn_item(ItemKind::Sword, pos)
            .expect("sword item");
        let floating = world
            .sandbox_spawn_item(ItemKind::Amulet, pos)
            .expect("amulet item");

        for _ in 0..30 {
            world.step(crate::input::TickInput::default());
        }

        let falling_y = world.store().get(falling).unwrap().body.pos.y;
        let floating_y = world.store().get(floating).unwrap().body.pos.y;
        assert!(falling_y > pos.y + 1.0, "falling item should drop");
        assert!(
            (floating_y - pos.y).abs() < 0.01,
            "floating item should stay in place"
        );
    }

    #[test]
    fn touch_item_is_auto_consumed_on_overlap() {
        let mut world = World::new_sandbox(crate::levels::SANDBOX_LEVEL).unwrap();
        let player_pos = world.store().player(0).unwrap().body.pos;
        // Give enough aura to satisfy potion requirements.
        if let Some(entity) = world.store_mut().player_mut(0) {
            entity.attributes.aura = 2;
            entity.stats = crate::entities::Stats::player(entity.attributes);
            let max_aura = entity.stats.max_aura;
            entity.health.current = (entity.stats.max_health - 1.0).max(1.0);
            if let Some(p) = entity.player_mut() {
                p.aura = max_aura;
            }
        }
        let health_before = world.store().player(0).unwrap().health.current;
        let potion_id = world
            .sandbox_spawn_item(ItemKind::Potion, player_pos)
            .expect("potion");

        world.step(crate::input::TickInput::default());

        assert!(world.store().get(potion_id).is_none(), "potion consumed");
        let health_after = world.store().player(0).unwrap().health.current;
        assert!(
            health_after > health_before,
            "potion should heal on touch pickup"
        );
    }

    #[test]
    fn interact_item_requires_action_key_and_equips_directly() {
        let mut world = World::new_sandbox(crate::levels::SANDBOX_LEVEL).unwrap();
        let player_pos = world.store().player(0).unwrap().body.pos;
        // Cap67 has no strict attribute barrier for pickup in this setup and equips to head.
        let item_id = world
            .sandbox_spawn_item(ItemKind::Cap67, player_pos)
            .expect("cap item");

        // No interact edge: item remains on the ground.
        world.step(crate::input::TickInput::default());
        assert!(world.store().get(item_id).is_some());
        assert!(world.store().player(0).unwrap().equipped[EquipSlot::Head.index()].is_none());

        // Interact edge: picked up and equipped directly (no inventory).
        let cmd = crate::input::PlayerCommand {
            interact: true,
            ..crate::input::PlayerCommand::none()
        };
        let input = crate::input::TickInput::single(cmd);
        world.step(input);

        assert!(world.store().get(item_id).is_none());
        let equipped = world.store().player(0).unwrap().equipped[EquipSlot::Head.index()];
        assert_eq!(equipped.map(|e| e.kind), Some(ItemKind::Cap67));
    }

    #[test]
    fn interact_pickup_prefers_higher_priority_when_items_overlap() {
        let mut world = World::new_sandbox(crate::levels::SANDBOX_LEVEL).unwrap();
        let player_pos = world.store().player(0).unwrap().body.pos;

        let mut low = ItemKind::Cap67.data();
        low.pickup = low.pickup.with_priority(0);
        let mut high = ItemKind::Shirt67.data();
        high.pickup = high.pickup.with_priority(10);

        let low_id = world.spawn_item(low, player_pos);
        let high_id = world.spawn_item(high, player_pos);

        let cmd = crate::input::PlayerCommand {
            interact: true,
            ..crate::input::PlayerCommand::none()
        };
        world.step(crate::input::TickInput::single(cmd));

        assert!(
            world.store().get(high_id).is_none(),
            "higher-priority item should be taken by action button"
        );
        assert!(
            world.store().get(low_id).is_some(),
            "lower-priority overlapping item should remain"
        );
        let equipped_armor = world.store().player(0).unwrap().equipped[EquipSlot::Armor.index()];
        assert_eq!(equipped_armor.map(|e| e.kind), Some(ItemKind::Shirt67));
    }
}
