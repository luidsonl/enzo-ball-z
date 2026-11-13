//! Orquestração do mundo e passo fixo.

use crate::ai;
use crate::aura;
use crate::entities::{
    EntityId, EntitySpec, EntityStore, EquipError, EquipSlot, ItemData, ItemKind,
    PlayableCharacter, PlayerId, VisualProfile, ENZO_SIZE, SLIME_SIZE,
};
use crate::events::{GameEvent, SkillRejectReason, TickOutput};
use crate::input::{PlayerCommand, TickInput};
use crate::map::TILE_SIZE;
use crate::map::{Level, LevelError};
use crate::math::Vec2;
use crate::mechanics::{self, Locomotion};
use crate::progression::{self, SaveData};
use crate::skills::{
    melee, mogging, nutella_shot, shout, LoadoutChange, LoadoutError, LoadoutRequest,
    SkillActivation, SkillId, SkillResult, SkillTiming,
};
use crate::snapshot::WorldSnapshot;
use crate::DT;

pub use crate::state::GamePhase;

/// Tempo entre a morte e o respawn automático.
pub const RESPAWN_DELAY: f32 = 1.2;

/// Distância abaixo do mapa em que a queda vira morte.
pub const FALL_DEATH_MARGIN: f32 = 64.0;

pub struct World {
    level: Level,
    store: EntityStore,
    tick: u64,
    phase: GamePhase,
    phase_timer: f32,
    pending_events: Vec<GameEvent>,
}

impl World {
    pub fn new(source: &str) -> Result<Self, LevelError> {
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

        Ok(Self {
            level,
            store,
            tick: 0,
            phase: GamePhase::Playing,
            phase_timer: 0.0,
            pending_events: Vec::new(),
        })
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

    pub fn add_inventory(&mut self, player_id: PlayerId, kind: ItemKind) -> bool {
        let Some(entity) = self.store.player_mut(player_id) else {
            return false;
        };
        let Some(player) = entity.player_mut() else {
            return false;
        };
        player.add_item(kind);
        true
    }

    /// Equips an item from the player's inventory, swapping any previously equipped item
    /// in the same slot back into inventory. Authoritative validation runs in core.
    pub fn equip_inventory_item(
        &mut self,
        player_id: PlayerId,
        kind: ItemKind,
    ) -> Result<(), EquipError> {
        let item = kind.data();
        let slot = item.slot().ok_or(EquipError::NotEquippable)?;
        // Validate first (no mutation).
        {
            let entity = self
                .store
                .player(player_id)
                .ok_or(EquipError::NotEquippable)?;
            item.can_be_equipped_by(entity.entity_type(), entity.character_attribute_set())?;
            let player = entity.player().ok_or(EquipError::NotEquippable)?;
            if !player.inventory.contains(&kind) {
                return Err(EquipError::NotEquippable);
            }
        }
        let entity = self
            .store
            .player_mut(player_id)
            .ok_or(EquipError::NotEquippable)?;
        {
            let player = entity.player_mut().ok_or(EquipError::NotEquippable)?;
            if !player.remove_item(kind) {
                return Err(EquipError::NotEquippable);
            }
        }
        if let Some(old) = entity.unequip(slot) {
            if let Some(player) = entity.player_mut() {
                player.add_item(old.kind);
            }
        }
        entity.try_equip(&item)?;
        self.pending_events.push(GameEvent::SaveRequested);
        Ok(())
    }

    /// Unequips a slot and returns the item to inventory.
    pub fn unequip_slot(&mut self, player_id: PlayerId, slot: EquipSlot) -> bool {
        let Some(entity) = self.store.player_mut(player_id) else {
            return false;
        };
        let Some(old) = entity.unequip(slot) else {
            return false;
        };
        if let Some(player) = entity.player_mut() {
            player.add_item(old.kind);
        }
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
                if progression::allocate_point(entity, attribute) {
                    events.push(GameEvent::AttributeAllocated {
                        player: entity_id,
                        attribute,
                    });
                }
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
                if progression::allocate_point(entity, attribute) {
                    events.push(GameEvent::AttributeAllocated {
                        player: entity_id,
                        attribute,
                    });
                }
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
            entity.skills.apply_intent(command, &entity.body);
            let speed_factor = entity.locomotion.speed_factor();
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

            if entity.is_player() && entity.body.pos.y > fall_limit {
                entity.health.current = 0.0;
            }
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

        // 10. Progressão
        progression::process_kills(&mut self.store, events);

        // 11. Limpeza
        let dead: Vec<EntityId> = self
            .store
            .enemies()
            .filter(|entity| !entity.health.is_alive())
            .map(|entity| entity.id)
            .collect();
        for id in dead {
            self.store.despawn(id);
        }

        let dead_players: Vec<EntityId> = self
            .store
            .players()
            .filter(|entity| !entity.health.is_alive())
            .map(|entity| entity.id)
            .collect();
        if !dead_players.is_empty() {
            for player in dead_players {
                events.push(GameEvent::PlayerDied { player });
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
            };

            let result = match skill {
                SkillId::Melee => {
                    let Some(entity) = self.store.get_mut(entity_id) else {
                        return;
                    };
                    let mut state = entity.skills.melee;
                    let r = melee::activate(&mut state, act, &mut self.store, events);
                    if let Some(entity) = self.store.get_mut(entity_id) {
                        entity.skills.melee = state;
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
                player.pending_level_up = false;
                player.respawn = spawn;
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
            .filter(|entity| entity.is_enemy() || entity.is_projectile())
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
        assert!(world.add_inventory(0, ItemKind::Cap67));
        world.equip_inventory_item(0, ItemKind::Cap67).unwrap();
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
    fn equip_inventory_item_moves_between_inventory_and_slots() {
        let mut world = World::new(LEVEL_1).unwrap();
        assert!(world.add_inventory(0, ItemKind::Cap67));
        assert!(world.add_inventory(0, ItemKind::Shirt67));
        world.equip_inventory_item(0, ItemKind::Cap67).unwrap();
        let entity = world.store().player(0).unwrap();
        assert!(entity.equipped[EquipSlot::Head.index()].is_some());
        assert!(!entity
            .player()
            .unwrap()
            .inventory
            .contains(&ItemKind::Cap67));

        // Swap: equip another head item? use Sword slot behavior by equipping shirt on armor slot.
        world.equip_inventory_item(0, ItemKind::Shirt67).unwrap();
        let entity = world.store().player(0).unwrap();
        assert!(entity.equipped[EquipSlot::Armor.index()].is_some());
    }

    #[test]
    fn unequip_slot_returns_item_to_inventory() {
        let mut world = World::new(LEVEL_1).unwrap();
        world.add_inventory(0, ItemKind::Sneakers67);
        world.equip_inventory_item(0, ItemKind::Sneakers67).unwrap();
        assert!(world.unequip_slot(0, EquipSlot::Feet));
        let entity = world.store().player(0).unwrap();
        assert!(entity.equipped[EquipSlot::Feet.index()].is_none());
        assert!(entity
            .player()
            .unwrap()
            .inventory
            .contains(&ItemKind::Sneakers67));
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
            outfit_bottom: 1,
            shoes: 2,
        };
        assert!(world.set_visual_profile(0, profile));
        let entity = world.store().player(0).unwrap();
        assert_eq!(entity.player().unwrap().visual_profile, profile);
        assert_eq!(entity.player().unwrap().character, PlayableCharacter::Enzo);
    }
}
