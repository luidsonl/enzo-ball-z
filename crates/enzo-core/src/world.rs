//! Orquestração do mundo e passo fixo.
//!
//! [`World::step`] é a única porta de avanço do tempo. A ordem dos sistemas é
//! parte do contrato (ver `docs/ARCHITECTURE.md`): input, aura, movimentos,
//! ataques, projéteis, IA, dano de contato, progressão, limpeza e eventos.

use crate::ai;
use crate::attacks::{self, melee, mogar};
use crate::aura;
use crate::entities::{
    Entity, EntityId, EntitySpec, EntityStore, EquipError, ItemData, PlayerId, ENZO_SIZE,
    SLIME_SIZE,
};
use crate::events::{GameEvent, TickOutput};
use crate::input::{PlayerCommand, TickInput};
use crate::map::{Level, LevelError};
use crate::math::Vec2;
use crate::progression::{self, SaveData};
use crate::snapshot::WorldSnapshot;
use crate::DT;

/// Tempo entre a morte e o respawn automático.
pub const RESPAWN_DELAY: f32 = 1.2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamePhase {
    Playing,
    LevelUp,
    Dead,
    Victory,
}

pub struct World {
    level: Level,
    store: EntityStore,
    tick: u64,
    phase: GamePhase,
    phase_timer: f32,
}

impl World {
    /// Cria a fase 1 a partir do mapa ASCII embutido no core.
    pub fn new(source: &str) -> Result<Self, LevelError> {
        let level = Level::parse(source)?;
        let mut store = EntityStore::new();

        let spawn = level
            .spawn_points(ENZO_SIZE)
            .first()
            .copied()
            .unwrap_or(Vec2::ZERO);
        store.spawn(EntitySpec::player(0, spawn));
        for pos in level.enemy_points(SLIME_SIZE) {
            store.spawn(EntitySpec::slime(pos));
        }

        Ok(Self {
            level,
            store,
            tick: 0,
            phase: GamePhase::Playing,
            phase_timer: 0.0,
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

    /// Equipa um item no jogador, validando as restrições no core.
    pub fn equip(&mut self, player_id: PlayerId, item: &ItemData) -> Result<(), EquipError> {
        let entity_id = self
            .store
            .player_entity_id(player_id)
            .ok_or(EquipError::NotEquippable)?;
        self.store.equip(entity_id, item)
    }

    pub fn save_data(&self) -> Option<SaveData> {
        self.store
            .player(0)
            .and_then(progression::SaveData::from_entity)
    }

    pub fn load_save(&mut self, save: &SaveData) -> bool {
        self.store
            .player_mut(0)
            .is_some_and(|entity| save.apply_to(entity))
    }

    /// Avança exatamente um tick de `1/60 s`.
    pub fn step(&mut self, input: TickInput) -> TickOutput {
        let mut events = Vec::new();

        // 1. Input: traduz comandos em intenção e edges.
        for index in 0..crate::MAX_PLAYERS {
            let Some(command) = input.command(index) else {
                continue;
            };
            let player_id = index as PlayerId;
            match self.phase {
                GamePhase::Playing => {
                    self.apply_playing_command(player_id, command);
                }
                GamePhase::LevelUp => {
                    self.apply_level_up_command(player_id, command, &mut events);
                }
                GamePhase::Dead | GamePhase::Victory => {
                    if command.restart {
                        self.respawn(&mut events);
                    }
                }
            }
        }

        match self.phase {
            GamePhase::Playing => self.step_playing(&mut events),
            GamePhase::Dead => {
                self.phase_timer = (self.phase_timer - DT).max(0.0);
                if self.phase_timer <= 0.0 {
                    self.respawn(&mut events);
                }
            }
            GamePhase::LevelUp | GamePhase::Victory => {}
        }

        self.clear_input_edges();
        self.tick = self.tick.wrapping_add(1);
        TickOutput::new(events)
    }

    fn apply_playing_command(&mut self, player_id: PlayerId, command: PlayerCommand) {
        let Some(entity) = self.store.player_mut(player_id) else {
            return;
        };
        if !entity.health.is_alive() {
            return;
        }

        if let Some(controller) = entity.controller.player_controller_mut() {
            controller.apply(command);
        }

        entity.movement.apply_command(command);
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
        // 2. Aura: consome o pedido do movimento SixSeven e aplica o ganho.
        // O componente SixSeven avança o tempo junto do movimento (etapa 3).
        for entity in self.store.players_mut() {
            let Entity {
                kind,
                movement,
                stats,
                ..
            } = entity;
            let Some(player) = kind.player_mut() else {
                continue;
            };
            let max_aura = stats.max_aura;
            if let Some(key) = movement.sixseven.take_request() {
                aura::apply_sixseven(player, &mut movement.sixseven, key, max_aura);
            }
        }

        // 3. Movimentos + 4. física: a intenção vira velocidade e o corpo é
        // integrado contra o tilemap. A lentidão do SixSeven é calculada pelo
        // próprio componente.
        let level = &self.level;
        for entity in self.store.iter_mut() {
            if !entity.is_actor() || !entity.health.is_alive() {
                continue;
            }
            entity
                .movement
                .update(&mut entity.body, &entity.stats, level, DT);
        }

        // 5. Combo físico e 6. disparo do Mogar.
        for index in 0..crate::MAX_PLAYERS {
            let player_id = index as PlayerId;
            if let Some(entity_id) = self.store.player_entity_id(player_id) {
                melee::update(&mut self.store, entity_id, events);
            }
            mogar::fire(&mut self.store, player_id, events);
        }

        // 7. Projéteis: movimento, dano e colisão com tiles.
        mogar::update_projectiles(&mut self.store, &self.level, DT, events);

        // 8. IA: cada inimigo decide sua intenção no componente `Ai`.
        ai::update(&mut self.store, &self.level);

        // 9. Colisões: dano de contato respeitando times e i-frames.
        attacks::contact_damage(&mut self.store, events);
        attacks::tick_timers(&mut self.store, DT);

        // 10. Progressão: XP, level-up e pedido de save.
        progression::process_kills(&mut self.store, events);

        // 11. Limpeza: inimigos mortos saem depois de conceder XP.
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

        if self.store.players().any(|entity| {
            entity
                .player()
                .is_some_and(|player| player.pending_level_up)
        }) {
            self.phase = GamePhase::LevelUp;
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
            }
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
        for pos in self.level.enemy_points(SLIME_SIZE) {
            self.store.spawn(EntitySpec::slime(pos));
        }

        self.phase = GamePhase::Playing;
        self.phase_timer = 0.0;
        events.push(GameEvent::PhaseRestarted { player: 0 });
    }

    pub fn snapshot(&self) -> WorldSnapshot {
        let entities = self
            .store
            .iter()
            .map(crate::snapshot::EntitySnapshot::from_entity);
        let camera_hint = self
            .store
            .player(0)
            .map(|entity| entity.body.pos)
            .unwrap_or(Vec2::ZERO);

        WorldSnapshot {
            version: crate::snapshot::SNAPSHOT_VERSION,
            tick: self.tick,
            phase: self.phase,
            phase_timer: self.phase_timer,
            entities: entities.collect(),
            level_width: self.level.width_px(),
            level_height: self.level.height_px(),
            flag: self.level.flag_rect(),
            camera_hint,
        }
    }
}
