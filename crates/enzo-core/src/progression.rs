//! Progressão: XP, nível, pontos, atributos e save.
//!
//! Modular progression operations keep point balance consistent:
//! `applied_points + unspent_points == total_points_for_level(level)`.

use crate::entities::{
    Attributes, Entity, EntityId, EntityStore, PlayableCharacter, Stats, VisualProfile,
};
use crate::events::GameEvent;
use crate::input::Attribute;
use crate::skills::Loadout;

pub const SAVE_VERSION: u16 = 2;
pub const SAVE_VERSION_V1: u16 = 1;
pub const POINTS_PER_LEVEL: u32 = 5;

/// XP necessário para sair do nível atual.
pub fn xp_to_next(level: u32) -> f32 {
    let level = level.max(1) as f32;
    50.0 * level * level + 100.0 * level
}

/// Returns total unspent+spent attribute points for a given level.
pub fn total_points_for_level(level: u32) -> u32 {
    level.saturating_sub(1).saturating_mul(POINTS_PER_LEVEL)
}

/// Sum of applied (spent) attribute points.
pub fn applied_points(attrs: Attributes) -> u32 {
    attrs
        .speed
        .saturating_add(attrs.strength)
        .saturating_add(attrs.health)
        .saturating_add(attrs.aura)
        .saturating_add(attrs.ego)
}

/// Consistent point balance for a player entity.
pub fn point_balance(entity: &Entity) -> Option<(u32, u32, u32)> {
    let player = entity.player()?;
    let applied = applied_points(entity.attributes);
    let unspent = player.points;
    let expected = total_points_for_level(player.level);
    Some((applied, unspent, expected))
}

/// Adiciona XP ao jogador e processa todos os level-ups alcançados.
pub fn grant_xp(entity: &mut Entity, amount: f32, events: &mut Vec<GameEvent>) {
    if amount <= 0.0 || !entity.health.is_alive() {
        return;
    }

    let entity_id = entity.id;
    let max_health = entity.stats.max_health;
    let max_aura = entity.stats.max_aura;

    let leveled = {
        let Some(player) = entity.player_mut() else {
            return;
        };
        player.xp += amount;
        let mut leveled = false;

        while player.xp >= xp_to_next(player.level) {
            player.xp -= xp_to_next(player.level);
            player.level += 1;
            player.points = player.points.saturating_add(POINTS_PER_LEVEL);
            player.pending_level_up = true;
            player.aura = max_aura;
            leveled = true;
            events.push(GameEvent::LevelUp {
                player: entity_id,
                level: player.level,
            });
        }

        leveled
    };

    if leveled {
        entity.health.current = max_health;
        events.push(GameEvent::SaveRequested);
    }
}

/// Concede XP de todos os inimigos mortos ao(s) jogador(es) vivo(s).
pub fn process_kills(store: &mut EntityStore, events: &mut Vec<GameEvent>) {
    let dead: Vec<(EntityId, f32)> = store
        .enemies()
        .filter(|entity| !entity.health.is_alive())
        .filter(|entity| entity.enemy().is_some_and(|data| !data.xp_awarded))
        .map(|entity| {
            let xp = entity.enemy().map(|data| data.xp_reward).unwrap_or(0.0);
            (entity.id, xp)
        })
        .collect();

    if dead.is_empty() {
        return;
    }

    let players: Vec<EntityId> = store.players().map(|entity| entity.id).collect();

    for (enemy_id, xp) in dead {
        if let Some(data) = store.get_mut(enemy_id).and_then(Entity::enemy_mut) {
            data.xp_awarded = true;
        }
        events.push(GameEvent::EnemyKilled {
            enemy: enemy_id,
            xp,
        });
        for player in &players {
            if let Some(entity) = store.get_mut(*player) {
                grant_xp(entity, xp, events);
            }
        }
    }
}

/// Spends one unspent point into an attribute (point assignment event).
pub fn allocate_point(
    entity: &mut Entity,
    attribute: Attribute,
    events: &mut Vec<GameEvent>,
) -> bool {
    let old_max_health = entity.stats.max_health;
    let old_max_aura = entity.stats.max_aura;

    let Some(points) = entity.player().map(|player| player.points) else {
        return false;
    };
    if points == 0 {
        return false;
    }

    entity.attributes.add(attribute);
    let stats = Stats::player(entity.attributes);

    let Some(player) = entity.player_mut() else {
        return false;
    };
    player.points -= 1;
    player.aura = (player.aura + (stats.max_aura - old_max_aura)).clamp(0.0, stats.max_aura);

    entity.stats = stats;
    entity.health.current =
        (entity.health.current + (stats.max_health - old_max_health)).min(stats.max_health);

    if let Some(player) = entity.player_mut() {
        if player.points == 0 {
            player.pending_level_up = false;
        }
    }

    events.push(GameEvent::AttributeAllocated {
        player: entity.id,
        attribute,
    });
    true
}

/// Fecha a tela de level-up. Pontos não gastos continuam disponíveis.
pub fn confirm_level_up(entity: &mut Entity) {
    if let Some(player) = entity.player_mut() {
        player.pending_level_up = false;
    }
}

/// Sandbox/debug: sets level and resets XP/attributes/points for a clean redistribute state.
/// Keeps character identity, inventory, equipment, and loadout; recomputes stats/HP/aura.
pub fn debug_set_level(entity: &mut Entity, level: u32) -> bool {
    let Some(player) = entity.player_mut() else {
        return false;
    };
    let level = level.max(1);
    player.level = level;
    player.xp = 0.0;
    player.points = total_points_for_level(level);
    player.pending_level_up = player.points > 0;
    player.aura = 0.0;

    entity.attributes = crate::entities::Attributes::default();
    let stats = Stats::player(entity.attributes);
    entity.stats = stats;
    entity.health.current = stats.max_health;
    if let Some(player) = entity.player_mut() {
        player.aura = stats.max_aura;
    }
    entity.skills.loadout.sanitize();
    true
}

/// Sandbox/debug: grants levels (adds points per level gained) without resetting attributes.
pub fn debug_grant_levels(entity: &mut Entity, levels: u32) -> bool {
    if levels == 0 {
        return entity.player().is_some();
    }
    let Some(player) = entity.player_mut() else {
        return false;
    };
    player.level = player.level.saturating_add(levels);
    player.points = player
        .points
        .saturating_add(levels.saturating_mul(POINTS_PER_LEVEL));
    player.pending_level_up = player.points > 0;
    player.xp = 0.0;
    let stats = entity.stats;
    if let Some(player) = entity.player_mut() {
        player.aura = player.aura.min(stats.max_aura);
    }
    entity.health.current = entity.health.current.min(stats.max_health);
    true
}

/// Death/respawn progression penalty (level down event): lose one level (min 1) and 5 points from balance.
/// Unspent points are reduced by 5 first. If there are not enough unspent points, the
/// remaining deficit is removed randomly (deterministically) from applied status points.
/// Applied status points can be reduced down to 0.
/// Emits `LevelDown` when the level changes.
pub fn apply_death_level_loss(
    entity: &mut Entity,
    rng: &mut crate::rng::Rng,
    events: &mut Vec<GameEvent>,
) -> bool {
    if entity.player().is_none() {
        return false;
    }

    let (old_level, old_unspent) = entity
        .player()
        .map(|p| (p.level.max(1), p.points))
        .unwrap_or((1, 0));
    let new_level = old_level.saturating_sub(1).max(1);
    let points_lost = if old_level > new_level {
        POINTS_PER_LEVEL
    } else {
        0
    };

    let unspent_after = old_unspent.saturating_sub(points_lost);
    let deficit_from_applied = points_lost.saturating_sub(old_unspent);

    if let Some(player) = entity.player_mut() {
        player.level = new_level;
        player.xp = 0.0;
        player.points = unspent_after;
        player.pending_level_up = player.points > 0;
    }

    let mut remaining = deficit_from_applied;
    while remaining > 0 {
        let Some(attr) = pick_attribute_to_decrement(entity.attributes, rng) else {
            break;
        };
        decrement_attribute(&mut entity.attributes, attr);
        remaining -= 1;
    }

    // Keep applied + unspent within the new level's total balance when possible.
    let new_total = total_points_for_level(new_level);
    let mut applied_after = applied_points(entity.attributes);
    let mut unspent_final = unspent_after;
    if applied_after.saturating_add(unspent_final) > new_total {
        let mut extra = applied_after.saturating_add(unspent_final) - new_total;
        while extra > 0 {
            let Some(attr) = pick_attribute_to_decrement(entity.attributes, rng) else {
                break;
            };
            decrement_attribute(&mut entity.attributes, attr);
            applied_after = applied_points(entity.attributes);
            if applied_after.saturating_add(unspent_final) <= new_total {
                break;
            }
            extra = extra.saturating_sub(1);
        }
        let max_unspent = new_total.saturating_sub(applied_points(entity.attributes));
        unspent_final = unspent_final.min(max_unspent);
        if let Some(player) = entity.player_mut() {
            player.points = unspent_final;
            player.pending_level_up = player.points > 0;
        }
    }

    let stats = Stats::player(entity.attributes);
    entity.stats = stats;
    entity.health.current = entity.health.current.min(stats.max_health).max(0.0);
    if let Some(player) = entity.player_mut() {
        player.aura = player.aura.min(stats.max_aura);
    }

    if old_level != new_level {
        events.push(GameEvent::LevelDown {
            player: entity.id,
            level: new_level,
        });
    }
    true
}

/// Picks an applied status attribute to remove one point from (can go down to 0).
fn pick_attribute_to_decrement(attrs: Attributes, rng: &mut crate::rng::Rng) -> Option<Attribute> {
    let available = removable_attributes(attrs, 0);
    if available.is_empty() {
        return None;
    }
    let idx = (rng.next_u64() as usize) % available.len();
    Some(available[idx])
}

/// Attributes that can lose one applied point while staying at/above `min_after`.
fn removable_attributes(attrs: Attributes, min_after: u32) -> Vec<Attribute> {
    let mut out = Vec::with_capacity(5);
    if attrs.speed > min_after {
        out.push(Attribute::Speed);
    }
    if attrs.strength > min_after {
        out.push(Attribute::Strength);
    }
    if attrs.health > min_after {
        out.push(Attribute::Health);
    }
    if attrs.aura > min_after {
        out.push(Attribute::Aura);
    }
    if attrs.ego > min_after {
        out.push(Attribute::Ego);
    }
    out
}

fn decrement_attribute(attrs: &mut Attributes, attr: Attribute) {
    match attr {
        Attribute::Speed => attrs.speed = attrs.speed.saturating_sub(1),
        Attribute::Strength => attrs.strength = attrs.strength.saturating_sub(1),
        Attribute::Health => attrs.health = attrs.health.saturating_sub(1),
        Attribute::Aura => attrs.aura = attrs.aura.saturating_sub(1),
        Attribute::Ego => attrs.ego = attrs.ego.saturating_sub(1),
    }
}

/// Sandbox/debug: reset allocated attributes and return all level points to unspent.
pub fn debug_reset_attributes(entity: &mut Entity) -> bool {
    let Some(player) = entity.player() else {
        return false;
    };
    debug_set_level(entity, player.level)
}

/// Persistent data. Position, enemies, and projectiles are not saved.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SaveData {
    pub version: u16,
    #[cfg_attr(feature = "serde", serde(default))]
    pub character: PlayableCharacter,
    #[cfg_attr(feature = "serde", serde(default))]
    pub visual_profile: VisualProfile,
    pub level: u32,
    pub xp: f32,
    pub points: u32,
    pub attributes: Attributes,
    #[cfg_attr(feature = "serde", serde(default))]
    pub loadout: Loadout,
}

impl SaveData {
    pub fn from_entity(entity: &Entity) -> Option<Self> {
        let player = entity.player()?;
        Some(Self {
            version: SAVE_VERSION,
            character: player.character,
            visual_profile: player.visual_profile,
            level: player.level,
            xp: player.xp,
            points: player.points,
            attributes: entity.attributes,
            loadout: entity.skills.loadout,
        })
    }

    /// Migra saves legados para a versão atual.
    pub fn migrate(mut self) -> Option<Self> {
        if self.version == SAVE_VERSION_V1 {
            self.version = SAVE_VERSION;
            self.loadout = Loadout::player_default();
        }
        if self.version != SAVE_VERSION {
            return None;
        }
        self.loadout.sanitize();
        Some(self)
    }

    /// Aplica o save se a versão for compatível (já migrado). Restaura vida/aura.
    pub fn apply_to(&self, entity: &mut Entity) -> bool {
        if self.version != SAVE_VERSION || entity.player().is_none() {
            return false;
        }

        entity.attributes = self.attributes;
        entity.visual_character = Some(self.character);
        entity.visual_profile = self.visual_profile;
        let stats = Stats::player(self.attributes);
        let Some(player) = entity.player_mut() else {
            return false;
        };
        player.character = self.character;
        player.visual_profile = self.visual_profile;
        player.level = self.level.max(1);
        player.xp = self.xp.max(0.0);
        player.points = self.points;
        player.aura = stats.max_aura;
        player.pending_level_up = player.points > 0;

        entity.stats = stats;
        entity.health.current = stats.max_health;
        let mut loadout = self.loadout;
        loadout.sanitize();
        entity.skills.loadout = loadout;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntitySpec;
    use crate::input::ActionSlot;
    use crate::math::Vec2;
    use crate::skills::SkillId;

    fn player() -> Entity {
        let mut store = EntityStore::new();
        let id = store.spawn(EntitySpec::player(0, Vec2::ZERO));
        store.despawn(id).unwrap()
    }

    #[test]
    fn xp_curve_scales_with_level() {
        assert_eq!(xp_to_next(1), 150.0);
        assert_eq!(xp_to_next(2), 400.0);
        assert_eq!(xp_to_next(3), 750.0);
    }

    #[test]
    fn level_up_grants_points_restores_and_pauses() {
        let mut entity = player();
        entity.health.current = 1.0;
        if let Some(data) = entity.player_mut() {
            data.aura = 0.0;
        }
        let mut events = Vec::new();

        grant_xp(&mut entity, 150.0, &mut events);

        let data = entity.player().unwrap();
        assert_eq!(data.level, 2);
        assert_eq!(data.xp, 0.0);
        assert_eq!(data.points, 5);
        assert!(data.pending_level_up);
        assert_eq!(data.aura, 10.0);
        assert_eq!(entity.health.current, 10.0);
        assert!(events
            .iter()
            .any(|event| matches!(event, GameEvent::LevelUp { level: 2, .. })));
    }

    #[test]
    fn large_xp_grants_multiple_levels() {
        let mut entity = player();
        let mut events = Vec::new();

        // 150 (lvl1) + 400 (lvl2) = 550 for two levels
        grant_xp(&mut entity, 550.0, &mut events);

        let data = entity.player().unwrap();
        assert_eq!(data.level, 3);
        assert_eq!(data.points, 10);
        assert_eq!(data.xp, 0.0);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, GameEvent::LevelUp { .. }))
                .count(),
            2
        );
    }

    #[test]
    fn allocating_speed_raises_walk_speed_and_emits_event() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        let mut events = Vec::new();
        assert!(allocate_point(&mut entity, Attribute::Speed, &mut events));
        assert_eq!(entity.stats.walk_speed, 230.0);
        assert_eq!(entity.player().unwrap().points, 0);
        assert!(!allocate_point(&mut entity, Attribute::Speed, &mut events));
        assert!(events.iter().any(|e| matches!(
            e,
            GameEvent::AttributeAllocated {
                attribute: Attribute::Speed,
                ..
            }
        )));
    }

    #[test]
    fn allocating_health_increases_max_and_heals_one() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        entity.health.current = 5.0;
        let mut events = Vec::new();
        assert!(allocate_point(&mut entity, Attribute::Health, &mut events));
        assert_eq!(entity.stats.max_health, 11.0);
        assert_eq!(entity.health.current, 6.0);
    }

    #[test]
    fn allocating_aura_raises_max_aura() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        let mut events = Vec::new();
        assert!(allocate_point(&mut entity, Attribute::Aura, &mut events));
        assert_eq!(entity.stats.max_aura, 11.0);
        assert_eq!(entity.stats.mogging_damage, 2.0);
    }

    #[test]
    fn allocating_ego_raises_mogging_damage() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        let mut events = Vec::new();
        assert!(allocate_point(&mut entity, Attribute::Ego, &mut events));
        assert_eq!(entity.stats.mogging_damage, 3.0);
    }

    #[test]
    fn allocating_strength_raises_melee_damage() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        let mut events = Vec::new();
        assert!(allocate_point(
            &mut entity,
            Attribute::Strength,
            &mut events
        ));
        assert_eq!(entity.stats.melee_damage(), 2.0);
    }

    #[test]
    fn save_roundtrip_preserves_progress_and_loadout() {
        let mut entity = player();
        {
            let data = entity.player_mut().unwrap();
            data.level = 4;
            data.xp = 42.0;
            data.points = 3;
        }
        entity.attributes = Attributes {
            speed: 1,
            strength: 2,
            health: 3,
            aura: 4,
            ego: 0,
        };
        entity.skills.loadout.clear(ActionSlot::Slot4);
        entity
            .skills
            .loadout
            .assign(ActionSlot::Slot4, SkillId::Shout);

        let save = SaveData::from_entity(&entity).unwrap();
        assert_eq!(save.version, SAVE_VERSION);

        let mut loaded = player();
        assert!(save.apply_to(&mut loaded));
        let data = loaded.player().unwrap();
        assert_eq!(data.level, 4);
        assert_eq!(data.xp, 42.0);
        assert_eq!(data.points, 3);
        assert_eq!(loaded.attributes, save.attributes);
        assert_eq!(loaded.stats, Stats::player(save.attributes));
        assert_eq!(
            loaded.skills.loadout.get(ActionSlot::Slot4),
            Some(SkillId::Shout)
        );
    }

    #[test]
    fn v1_save_migrates_to_default_loadout() {
        let save = SaveData {
            version: SAVE_VERSION_V1,
            character: Default::default(),
            visual_profile: Default::default(),
            level: 2,
            xp: 10.0,
            points: 1,
            attributes: Attributes::default(),
            loadout: Loadout::EMPTY,
        }
        .migrate()
        .unwrap();
        assert_eq!(save.version, SAVE_VERSION);
        assert_eq!(save.loadout, Loadout::player_default());
    }

    #[test]
    fn incompatible_save_is_ignored() {
        let mut entity = player();
        let save = SaveData {
            version: 999,
            character: Default::default(),
            visual_profile: Default::default(),
            level: 9,
            xp: 0.0,
            points: 9,
            attributes: Attributes::default(),
            loadout: Loadout::player_default(),
        };
        assert!(!save.apply_to(&mut entity));
        assert_eq!(entity.player().unwrap().level, 1);
    }

    #[test]
    fn process_kills_awards_xp_once() {
        let mut store = EntityStore::new();
        let player_id = store.spawn(EntitySpec::player(0, Vec2::ZERO));
        let enemy = store.spawn(EntitySpec::slime(Vec2::new(100.0, 0.0)));
        store.get_mut(enemy).unwrap().health.current = 0.0;
        let mut events = Vec::new();

        process_kills(&mut store, &mut events);
        process_kills(&mut store, &mut events);

        assert_eq!(store.get(player_id).unwrap().player().unwrap().xp, 25.0);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, GameEvent::EnemyKilled { .. }))
                .count(),
            1
        );
    }

    #[test]
    fn debug_set_level_resets_attributes_and_grants_unspent_points() {
        let mut entity = player();
        entity.attributes.speed = 3;
        entity.attributes.strength = 2;
        entity.player_mut().unwrap().xp = 77.0;
        assert!(debug_set_level(&mut entity, 4));
        let p = entity.player().unwrap();
        assert_eq!(p.level, 4);
        assert_eq!(p.xp, 0.0);
        assert_eq!(p.points, 15);
        assert!(p.pending_level_up);
        assert_eq!(entity.attributes, Attributes::default());
        assert_eq!(entity.stats, Stats::player(Attributes::default()));
        assert_eq!(entity.health.current, entity.stats.max_health);
        assert_eq!(p.aura, entity.stats.max_aura);
    }

    #[test]
    fn sandbox_death_level_loss_reduces_unspent_points_first() {
        // Lose 1 level and 5 points from balance. Unspent points are reduced by 5.
        let mut entity = player();
        assert!(debug_set_level(&mut entity, 5)); // 20 total
        let mut events = Vec::new();
        assert_eq!(entity.player().unwrap().points, 20);
        assert!(apply_death_level_loss(
            &mut entity,
            &mut crate::rng::Rng::new(1),
            &mut events
        ));
        let p = entity.player().unwrap();
        assert_eq!(p.level, 4);
        assert_eq!(p.points, 15);
        assert_eq!(p.xp, 0.0);
        assert_eq!(applied_points(entity.attributes), 0);
        assert_eq!(point_balance(&entity), Some((0, 15, 15)));
        assert!(events
            .iter()
            .any(|e| matches!(e, GameEvent::LevelDown { level: 4, .. })));

        // If not enough unspent points, remaining deficit is taken from applied status points.
        let mut entity = player();
        assert!(debug_set_level(&mut entity, 5)); // 20 total
        let mut events = Vec::new();
        for attr in [
            Attribute::Speed,
            Attribute::Speed,
            Attribute::Strength,
            Attribute::Strength,
            Attribute::Strength,
            Attribute::Health,
            Attribute::Health,
            Attribute::Aura,
            Attribute::Aura,
            Attribute::Ego,
        ] {
            assert!(allocate_point(&mut entity, attr, &mut events));
        }
        // 10 applied, 10 unspent
        assert_eq!(entity.player().unwrap().points, 10);
        assert_eq!(applied_points(entity.attributes), 10);
        assert!(apply_death_level_loss(
            &mut entity,
            &mut crate::rng::Rng::new(2),
            &mut events
        ));
        let p = entity.player().unwrap();
        assert_eq!(p.level, 4);
        assert_eq!(p.points, 5); // 10 - 5
        assert_eq!(applied_points(entity.attributes), 10); // no deficit from applied
        assert_eq!(point_balance(&entity), Some((10, 5, 15)));

        // No unspent points: all 5 taken from applied status points (can go down to 0).
        let mut entity = player();
        assert!(debug_set_level(&mut entity, 5)); // 20 total
        let mut events = Vec::new();
        for attr in [
            Attribute::Speed,
            Attribute::Strength,
            Attribute::Health,
            Attribute::Aura,
            Attribute::Ego,
        ] {
            assert!(allocate_point(&mut entity, attr, &mut events));
        }
        assert_eq!(entity.player().unwrap().points, 15);
        assert_eq!(applied_points(entity.attributes), 5);
        // Spend remaining unspent to have 0 unspent and 20 applied? easier: set up directly
        let mut entity = player();
        assert!(debug_set_level(&mut entity, 2)); // 5 total
        let mut events = Vec::new();
        for attr in [
            Attribute::Speed,
            Attribute::Strength,
            Attribute::Health,
            Attribute::Aura,
            Attribute::Ego,
        ] {
            assert!(allocate_point(&mut entity, attr, &mut events));
        }
        assert_eq!(entity.player().unwrap().points, 0);
        assert_eq!(applied_points(entity.attributes), 5);
        assert!(apply_death_level_loss(
            &mut entity,
            &mut crate::rng::Rng::new(3),
            &mut events
        ));
        let p = entity.player().unwrap();
        assert_eq!(p.level, 1);
        assert_eq!(p.points, 0);
        assert_eq!(applied_points(entity.attributes), 0);
        assert_eq!(point_balance(&entity), Some((0, 0, 0)));

        // Partial unspent: reduce unspent and take remainder from applied.
        let mut entity = player();
        assert!(debug_set_level(&mut entity, 3)); // 10 total
        let mut events = Vec::new();
        for attr in [
            Attribute::Speed,
            Attribute::Strength,
            Attribute::Health,
            Attribute::Aura,
            Attribute::Ego,
            Attribute::Speed,
            Attribute::Strength,
            Attribute::Health,
        ] {
            assert!(allocate_point(&mut entity, attr, &mut events));
        }
        // 8 applied, 2 unspent
        assert_eq!(entity.player().unwrap().points, 2);
        assert_eq!(applied_points(entity.attributes), 8);
        assert!(apply_death_level_loss(
            &mut entity,
            &mut crate::rng::Rng::new(4),
            &mut events
        ));
        let p = entity.player().unwrap();
        assert_eq!(p.level, 2);
        assert_eq!(p.points, 0); // 2 unspent consumed, 3 deficit from applied
        assert_eq!(applied_points(entity.attributes), 5);
        assert_eq!(point_balance(&entity), Some((5, 0, 5)));
    }

    #[test]
    fn sandbox_death_level_loss_is_deterministic_for_same_rng_seed() {
        let mut a = player();
        let mut b = player();
        assert!(debug_set_level(&mut a, 5));
        assert!(debug_set_level(&mut b, 5));
        let mut events = Vec::new();
        for attr in [
            Attribute::Speed,
            Attribute::Speed,
            Attribute::Strength,
            Attribute::Strength,
            Attribute::Health,
            Attribute::Health,
            Attribute::Aura,
            Attribute::Aura,
            Attribute::Ego,
            Attribute::Ego,
        ] {
            assert!(allocate_point(&mut a, attr, &mut events));
            assert!(allocate_point(&mut b, attr, &mut events));
        }
        let mut events_a = Vec::new();
        let mut events_b = Vec::new();
        assert!(apply_death_level_loss(
            &mut a,
            &mut crate::rng::Rng::new(123),
            &mut events_a
        ));
        assert!(apply_death_level_loss(
            &mut b,
            &mut crate::rng::Rng::new(123),
            &mut events_b
        ));
        assert_eq!(a.player().unwrap().level, b.player().unwrap().level);
        assert_eq!(a.attributes, b.attributes);
        assert_eq!(a.player().unwrap().points, b.player().unwrap().points);
        assert_eq!(point_balance(&a), point_balance(&b));
    }

    #[test]
    fn debug_grant_levels_adds_points_without_resetting_attributes() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        let mut events = Vec::new();
        assert!(allocate_point(
            &mut entity,
            Attribute::Strength,
            &mut events
        ));
        assert!(debug_grant_levels(&mut entity, 2));
        let p = entity.player().unwrap();
        assert_eq!(p.level, 3);
        assert_eq!(p.points, 10);
        assert_eq!(entity.attributes.strength, 1);
        assert_eq!(point_balance(&entity), Some((1, 10, 10)));
    }
}
