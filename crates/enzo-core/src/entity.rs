//! Modelo padronizado de entidades.
//!
//! Jogador, inimigo e projétil são o mesmo [`Entity`]. Os atributos comuns
//! (`Body`, `Health`, `Stats`, `Faction`, `Combat`) ficam sempre nos mesmos
//! componentes; o que muda é o [`EntityKind`].
//!
//! Toda criação passa por [`EntityStore::spawn`] com um [`EntitySpec`].

use std::collections::BTreeMap;

use crate::input::{Attribute, FarmKey};
use crate::math::{Rect, Vec2};

pub type PlayerId = u32;

pub const PLAYER_SIZE: Vec2 = Vec2::new(24.0, 30.0);
pub const ENEMY_SIZE: Vec2 = Vec2::new(26.0, 26.0);
pub const MOGAR_SIZE: Vec2 = Vec2::new(24.0, 12.0);
pub const MOGAR_SPEED: f32 = 620.0;
pub const MOGAR_LIFETIME: f32 = 1.4;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityId(pub u32);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Faction {
    Players,
    Enemies,
    Neutral,
}

impl Faction {
    pub fn hostile_to(self, other: Self) -> bool {
        matches!(
            (self, other),
            (Faction::Players, Faction::Enemies) | (Faction::Enemies, Faction::Players)
        )
    }
}

/// Pontos investidos em cada atributo. Jogador distribui no level-up; inimigos
/// usam valores fixos.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attributes {
    pub speed: u32,
    pub strength: u32,
    pub health: u32,
    pub aura: u32,
}

impl Attributes {
    pub fn get(self, attribute: Attribute) -> u32 {
        match attribute {
            Attribute::Speed => self.speed,
            Attribute::Strength => self.strength,
            Attribute::Health => self.health,
            Attribute::Aura => self.aura,
        }
    }

    pub fn add(&mut self, attribute: Attribute) {
        match attribute {
            Attribute::Speed => self.speed += 1,
            Attribute::Strength => self.strength += 1,
            Attribute::Health => self.health += 1,
            Attribute::Aura => self.aura += 1,
        }
    }
}

/// Atributos derivados, iguais para qualquer entidade.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub walk_speed: f32,
    pub run_speed: f32,
    pub jump_speed: f32,
    pub strength: f32,
    pub max_health: f32,
    pub max_aura: f32,
    pub mogar_damage: f32,
}

impl Stats {
    pub fn player(attributes: Attributes) -> Self {
        let speed = attributes.speed as f32;
        Self {
            walk_speed: 220.0 + 10.0 * speed,
            run_speed: 340.0 + 6.0 * speed,
            jump_speed: 560.0 + 2.0 * speed,
            strength: 2.0 * attributes.strength as f32,
            max_health: 100.0 + 10.0 * attributes.health as f32,
            max_aura: 100.0 + 10.0 * attributes.aura as f32,
            mogar_damage: 20.0 + 2.0 * attributes.aura as f32,
        }
    }

    pub fn enemy() -> Self {
        Self {
            walk_speed: 60.0,
            run_speed: 60.0,
            jump_speed: 0.0,
            strength: 0.0,
            max_health: 30.0,
            max_aura: 0.0,
            mogar_damage: 0.0,
        }
    }

    pub fn projectile() -> Self {
        Self {
            walk_speed: 0.0,
            run_speed: 0.0,
            jump_speed: 0.0,
            strength: 0.0,
            max_health: 1.0,
            max_aura: 0.0,
            mogar_damage: 0.0,
        }
    }

    /// `hit_index` é 0, 1 ou 2 (primeiro, segundo e terceiro golpe).
    pub fn melee_damage(&self, hit_index: usize) -> f32 {
        let base = if hit_index >= 2 { 12.0 } else { 6.0 };
        base + self.strength
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Body {
    pub pos: Vec2,
    pub vel: Vec2,
    pub size: Vec2,
    pub on_ground: bool,
    pub facing: i8,
}

impl Body {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            pos,
            vel: Vec2::ZERO,
            size,
            on_ground: false,
            facing: 1,
        }
    }

    pub fn rect(&self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }

    pub fn center(&self) -> Vec2 {
        self.rect().center()
    }

    pub fn bottom(&self) -> f32 {
        self.pos.y + self.size.y
    }

    pub fn right(&self) -> f32 {
        self.pos.x + self.size.x
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Health {
    pub current: f32,
    pub iframes: f32,
}

impl Health {
    pub fn full(max: f32) -> Self {
        Self {
            current: max,
            iframes: 0.0,
        }
    }

    pub fn is_alive(self) -> bool {
        self.current > 0.0
    }

    /// Aplica dano e retorna o valor efetivamente descontado.
    pub fn damage(&mut self, amount: f32) -> f32 {
        let applied = amount.max(0.0).min(self.current);
        self.current -= applied;
        applied
    }

    pub fn heal(&mut self, amount: f32, max: f32) {
        self.current = (self.current + amount.max(0.0)).min(max);
    }

    pub fn tick(&mut self, dt: f32) {
        self.iframes = (self.iframes - dt).max(0.0);
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Combat {
    pub melee_step: u8,
    pub melee_window: f32,
    pub attack_cooldown: f32,
    pub attack_timer: f32,
    pub mogar_cooldown: f32,
}

impl Combat {
    pub fn tick(&mut self, dt: f32) {
        self.melee_window = (self.melee_window - dt).max(0.0);
        if self.melee_window <= 0.0 {
            self.melee_step = 0;
        }
        self.attack_cooldown = (self.attack_cooldown - dt).max(0.0);
        self.attack_timer = (self.attack_timer - dt).max(0.0);
        self.mogar_cooldown = (self.mogar_cooldown - dt).max(0.0);
    }

    pub fn can_attack(self) -> bool {
        self.attack_cooldown <= 0.0
    }

    pub fn can_fire_mogar(self) -> bool {
        self.mogar_cooldown <= 0.0
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AuraFarmState {
    pub last_key: Option<FarmKey>,
    pub combo: u32,
    pub window: f32,
    pub total: f32,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerInput {
    pub move_x: i8,
    pub jump_held: bool,
    pub melee_pressed: bool,
    pub mogar_pressed: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerData {
    pub player_id: PlayerId,
    pub level: u32,
    pub xp: f32,
    pub points: u32,
    pub attributes: Attributes,
    pub aura: f32,
    pub farm: AuraFarmState,
    pub input: PlayerInput,
    pub running: bool,
    pub last_move_x: i8,
    pub last_tap_dir: i8,
    pub tap_timer: f32,
    pub jump_buffer: f32,
    pub coyote: f32,
    pub jump_cut: bool,
    pub pending_level_up: bool,
    pub respawn: Vec2,
}

impl PlayerData {
    pub fn new(player_id: PlayerId, respawn: Vec2, stats: Stats) -> Self {
        Self {
            player_id,
            level: 1,
            xp: 0.0,
            points: 0,
            attributes: Attributes::default(),
            aura: stats.max_aura,
            farm: AuraFarmState::default(),
            input: PlayerInput::default(),
            running: false,
            last_move_x: 0,
            last_tap_dir: 0,
            tap_timer: 0.0,
            jump_buffer: 0.0,
            coyote: 0.0,
            jump_cut: false,
            pending_level_up: false,
            respawn,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyData {
    pub patrol_dir: i8,
    pub xp_reward: f32,
    pub contact_damage: f32,
    pub stun: f32,
}

impl Default for EnemyData {
    fn default() -> Self {
        Self {
            patrol_dir: -1,
            xp_reward: 25.0,
            contact_damage: 10.0,
            stun: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectileData {
    pub owner: EntityId,
    pub damage: f32,
    pub hits: Vec<EntityId>,
    pub lifetime: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum EntityKind {
    Player(PlayerData),
    Enemy(EnemyData),
    Projectile(ProjectileData),
}

impl EntityKind {
    pub fn is_player(&self) -> bool {
        matches!(self, Self::Player(_))
    }

    pub fn is_enemy(&self) -> bool {
        matches!(self, Self::Enemy(_))
    }

    pub fn is_projectile(&self) -> bool {
        matches!(self, Self::Projectile(_))
    }

    pub fn player(&self) -> Option<&PlayerData> {
        match self {
            Self::Player(data) => Some(data),
            _ => None,
        }
    }

    pub fn player_mut(&mut self) -> Option<&mut PlayerData> {
        match self {
            Self::Player(data) => Some(data),
            _ => None,
        }
    }

    pub fn enemy(&self) -> Option<&EnemyData> {
        match self {
            Self::Enemy(data) => Some(data),
            _ => None,
        }
    }

    pub fn enemy_mut(&mut self) -> Option<&mut EnemyData> {
        match self {
            Self::Enemy(data) => Some(data),
            _ => None,
        }
    }

    pub fn projectile(&self) -> Option<&ProjectileData> {
        match self {
            Self::Projectile(data) => Some(data),
            _ => None,
        }
    }

    pub fn projectile_mut(&mut self) -> Option<&mut ProjectileData> {
        match self {
            Self::Projectile(data) => Some(data),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,
    pub body: Body,
    pub health: Health,
    pub stats: Stats,
    pub faction: Faction,
    pub combat: Combat,
}

impl Entity {
    pub fn is_player(&self) -> bool {
        self.kind.is_player()
    }

    pub fn is_enemy(&self) -> bool {
        self.kind.is_enemy()
    }

    pub fn is_projectile(&self) -> bool {
        self.kind.is_projectile()
    }

    pub fn is_actor(&self) -> bool {
        self.is_player() || self.is_enemy()
    }

    pub fn player(&self) -> Option<&PlayerData> {
        self.kind.player()
    }

    pub fn player_mut(&mut self) -> Option<&mut PlayerData> {
        self.kind.player_mut()
    }

    pub fn enemy(&self) -> Option<&EnemyData> {
        self.kind.enemy()
    }

    pub fn enemy_mut(&mut self) -> Option<&mut EnemyData> {
        self.kind.enemy_mut()
    }

    pub fn projectile(&self) -> Option<&ProjectileData> {
        self.kind.projectile()
    }

    pub fn projectile_mut(&mut self) -> Option<&mut ProjectileData> {
        self.kind.projectile_mut()
    }

    pub fn max_health(&self) -> f32 {
        self.stats.max_health
    }

    pub fn max_aura(&self) -> f32 {
        self.stats.max_aura
    }

    pub fn aura(&self) -> f32 {
        self.player().map(|player| player.aura).unwrap_or(0.0)
    }

    pub fn rect(&self) -> Rect {
        self.body.rect()
    }

    pub fn center(&self) -> Vec2 {
        self.body.center()
    }
}

/// Especificação de criação. Use os construtores; não monte `Entity` à mão.
#[derive(Clone, Debug, PartialEq)]
pub struct EntitySpec {
    pub kind: EntityKind,
    pub pos: Vec2,
    pub size: Vec2,
    pub faction: Faction,
    pub health: Health,
    pub stats: Stats,
    pub velocity: Vec2,
}

impl EntitySpec {
    pub fn player(player_id: PlayerId, pos: Vec2) -> Self {
        let stats = Stats::player(Attributes::default());
        Self {
            kind: EntityKind::Player(PlayerData::new(player_id, pos, stats)),
            pos,
            size: PLAYER_SIZE,
            faction: Faction::Players,
            health: Health::full(stats.max_health),
            stats,
            velocity: Vec2::ZERO,
        }
    }

    pub fn enemy(pos: Vec2) -> Self {
        let stats = Stats::enemy();
        Self {
            kind: EntityKind::Enemy(EnemyData::default()),
            pos,
            size: ENEMY_SIZE,
            faction: Faction::Enemies,
            health: Health::full(stats.max_health),
            stats,
            velocity: Vec2::new(-stats.walk_speed, 0.0),
        }
    }

    pub fn mogar(owner: EntityId, pos: Vec2, facing: i8, damage: f32) -> Self {
        let stats = Stats::projectile();
        Self {
            kind: EntityKind::Projectile(ProjectileData {
                owner,
                damage,
                hits: Vec::new(),
                lifetime: MOGAR_LIFETIME,
            }),
            pos,
            size: MOGAR_SIZE,
            faction: Faction::Players,
            health: Health::full(stats.max_health),
            stats,
            velocity: Vec2::new(if facing >= 0 { 1.0 } else { -1.0 } * MOGAR_SPEED, 0.0),
        }
    }

    pub fn with_health(mut self, current: f32) -> Self {
        self.health.current = current.clamp(0.0, self.stats.max_health);
        self
    }

    pub fn with_velocity(mut self, velocity: Vec2) -> Self {
        self.velocity = velocity;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntityStore {
    next_id: u32,
    entities: BTreeMap<EntityId, Entity>,
}

impl EntityStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn spawn(&mut self, spec: EntitySpec) -> EntityId {
        let id = EntityId(self.next_id);
        self.next_id = self.next_id.wrapping_add(1);

        let mut body = Body::new(spec.pos, spec.size);
        body.vel = spec.velocity;

        let mut health = spec.health;
        health.current = health.current.clamp(0.0, spec.stats.max_health);

        let entity = Entity {
            id,
            kind: spec.kind,
            body,
            health,
            stats: spec.stats,
            faction: spec.faction,
            combat: Combat::default(),
        };
        self.entities.insert(id, entity);
        id
    }

    pub fn despawn(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    pub fn get(&self, id: EntityId) -> Option<&Entity> {
        self.entities.get(&id)
    }

    pub fn get_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.get_mut(&id)
    }

    pub fn len(&self) -> usize {
        self.entities.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &Entity> + '_ {
        self.entities.values()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut Entity> + '_ {
        self.entities.values_mut()
    }

    pub fn ids(&self) -> Vec<EntityId> {
        self.entities.keys().copied().collect()
    }

    pub fn players(&self) -> impl Iterator<Item = &Entity> + '_ {
        self.entities.values().filter(|entity| entity.is_player())
    }

    pub fn players_mut(&mut self) -> impl Iterator<Item = &mut Entity> + '_ {
        self.entities
            .values_mut()
            .filter(|entity| entity.is_player())
    }

    pub fn enemies(&self) -> impl Iterator<Item = &Entity> + '_ {
        self.entities.values().filter(|entity| entity.is_enemy())
    }

    pub fn enemies_mut(&mut self) -> impl Iterator<Item = &mut Entity> + '_ {
        self.entities
            .values_mut()
            .filter(|entity| entity.is_enemy())
    }

    pub fn actors(&self) -> impl Iterator<Item = &Entity> + '_ {
        self.entities.values().filter(|entity| entity.is_actor())
    }

    pub fn player(&self, player_id: PlayerId) -> Option<&Entity> {
        self.players().find(|entity| {
            entity
                .player()
                .is_some_and(|data| data.player_id == player_id)
        })
    }

    pub fn player_mut(&mut self, player_id: PlayerId) -> Option<&mut Entity> {
        self.players_mut().find(|entity| {
            entity
                .player()
                .is_some_and(|data| data.player_id == player_id)
        })
    }

    pub fn player_entity_id(&self, player_id: PlayerId) -> Option<EntityId> {
        self.player(player_id).map(|entity| entity.id)
    }

    pub fn in_rect(&self, rect: Rect, faction: Faction) -> Vec<EntityId> {
        self.entities
            .values()
            .filter(|entity| entity.faction == faction && entity.rect().overlaps(rect))
            .map(|entity| entity.id)
            .collect()
    }

    pub fn hostiles_in_rect(&self, rect: Rect, faction: Faction) -> Vec<EntityId> {
        self.entities
            .values()
            .filter(|entity| entity.faction.hostile_to(faction) && entity.rect().overlaps(rect))
            .map(|entity| entity.id)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn store_with_actors() -> (EntityStore, EntityId, EntityId, EntityId) {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let enemy = store.spawn(EntitySpec::enemy(Vec2::new(100.0, 0.0)));
        let mogar = store.spawn(EntitySpec::mogar(player, Vec2::new(200.0, 10.0), 1, 20.0));
        (store, player, enemy, mogar)
    }

    #[test]
    fn player_and_enemy_share_components() {
        let (store, player_id, enemy_id, _) = store_with_actors();
        let player = store.get(player_id).unwrap();
        let enemy = store.get(enemy_id).unwrap();

        assert_eq!(player.faction, Faction::Players);
        assert_eq!(enemy.faction, Faction::Enemies);
        assert_eq!(player.stats.max_health, 100.0);
        assert_eq!(enemy.stats.max_health, 30.0);
        assert_eq!(player.body.size, PLAYER_SIZE);
        assert_eq!(enemy.body.size, ENEMY_SIZE);
        assert_eq!(player.health.current, 100.0);
        assert_eq!(enemy.health.current, 30.0);
        assert!(player.is_actor());
        assert!(enemy.is_actor());
    }

    #[test]
    fn ids_are_sequential_and_stable() {
        let (store, player, enemy, mogar) = store_with_actors();
        assert_eq!(player, EntityId(0));
        assert_eq!(enemy, EntityId(1));
        assert_eq!(mogar, EntityId(2));
        assert_eq!(store.player_entity_id(0), Some(player));
        assert_eq!(store.player(0).unwrap().id, player);
    }

    #[test]
    fn mogar_carries_owner_and_velocity() {
        let (store, player, _, mogar) = store_with_actors();
        let projectile = store.get(mogar).unwrap();
        let data = projectile.projectile().unwrap();
        assert_eq!(data.owner, player);
        assert_eq!(data.damage, 20.0);
        assert_eq!(projectile.body.vel.x, MOGAR_SPEED);
        assert_eq!(projectile.faction, Faction::Players);
    }

    #[test]
    fn despawn_removes_entity() {
        let (mut store, _, enemy, _) = store_with_actors();
        assert!(store.despawn(enemy).is_some());
        assert!(store.get(enemy).is_none());
        assert!(store.despawn(enemy).is_none());
    }

    #[test]
    fn in_rect_filters_by_faction() {
        let (store, player, enemy, _) = store_with_actors();
        let around_player = Rect::new(-10.0, -10.0, 50.0, 50.0);
        assert_eq!(store.in_rect(around_player, Faction::Players), vec![player]);
        assert!(store.in_rect(around_player, Faction::Enemies).is_empty());
        let around_enemy = Rect::new(90.0, -10.0, 40.0, 40.0);
        assert_eq!(store.in_rect(around_enemy, Faction::Enemies), vec![enemy]);
    }

    #[test]
    fn hostiles_in_rect_uses_faction_relation() {
        let (store, player, enemy, _) = store_with_actors();
        let all = Rect::new(-10.0, -10.0, 200.0, 50.0);
        assert_eq!(store.hostiles_in_rect(all, Faction::Players), vec![enemy]);
        assert_eq!(store.hostiles_in_rect(all, Faction::Enemies), vec![player]);
    }

    #[test]
    fn health_damage_and_heal_clamp() {
        let mut health = Health::full(30.0);
        assert_eq!(health.damage(10.0), 10.0);
        assert_eq!(health.current, 20.0);
        assert_eq!(health.damage(100.0), 20.0);
        assert_eq!(health.current, 0.0);
        assert!(!health.is_alive());
        health.heal(50.0, 30.0);
        assert_eq!(health.current, 30.0);
    }

    #[test]
    fn stats_scale_with_attributes() {
        let attributes = Attributes {
            speed: 2,
            strength: 3,
            health: 1,
            aura: 4,
        };
        let stats = Stats::player(attributes);
        assert_eq!(stats.walk_speed, 240.0);
        assert_eq!(stats.run_speed, 352.0);
        assert_eq!(stats.jump_speed, 564.0);
        assert_eq!(stats.strength, 6.0);
        assert_eq!(stats.max_health, 110.0);
        assert_eq!(stats.max_aura, 140.0);
        assert_eq!(stats.mogar_damage, 28.0);
        assert_eq!(stats.melee_damage(0), 12.0);
        assert_eq!(stats.melee_damage(2), 18.0);
    }
}
