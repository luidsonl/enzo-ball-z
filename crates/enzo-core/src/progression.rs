//! Progressão: XP, nível, pontos, atributos e save.
//!
//! O core decide quanto XP sobe de nível e como cada ponto altera os stats. O
//! cliente apenas persiste o [`SaveData`] no `localStorage`.

use crate::entities::{Attributes, Entity, EntityId, EntityStore, Stats};
use crate::events::GameEvent;
use crate::input::Attribute;

pub const SAVE_VERSION: u16 = 1;

/// XP necessário para sair do nível atual.
pub fn xp_to_next(level: u32) -> f32 {
    100.0 * level.max(1) as f32
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
            player.points += 1;
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

/// Gasta um ponto no atributo escolhido e recalcula os stats derivados.
pub fn allocate_point(entity: &mut Entity, attribute: Attribute) -> bool {
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
    true
}

/// Fecha a tela de level-up. Pontos não gastos continuam disponíveis.
pub fn confirm_level_up(entity: &mut Entity) {
    if let Some(player) = entity.player_mut() {
        player.pending_level_up = false;
    }
}

/// Dados persistentes. Posição, inimigos e projéteis não entram no save.
#[derive(Clone, Copy, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SaveData {
    pub version: u16,
    pub level: u32,
    pub xp: f32,
    pub points: u32,
    pub attributes: Attributes,
}

impl SaveData {
    pub fn from_entity(entity: &Entity) -> Option<Self> {
        let player = entity.player()?;
        Some(Self {
            version: SAVE_VERSION,
            level: player.level,
            xp: player.xp,
            points: player.points,
            attributes: entity.attributes,
        })
    }

    /// Aplica o save se a versão for compatível. Restaura vida e aura cheias.
    pub fn apply_to(&self, entity: &mut Entity) -> bool {
        if self.version != SAVE_VERSION || entity.player().is_none() {
            return false;
        }

        entity.attributes = self.attributes;
        let stats = Stats::player(self.attributes);
        let Some(player) = entity.player_mut() else {
            return false;
        };
        player.level = self.level.max(1);
        player.xp = self.xp.max(0.0);
        player.points = self.points;
        player.aura = stats.max_aura;
        player.pending_level_up = false;

        entity.stats = stats;
        entity.health.current = stats.max_health;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntitySpec;
    use crate::math::Vec2;

    fn player() -> Entity {
        let mut store = EntityStore::new();
        let id = store.spawn(EntitySpec::player(0, Vec2::ZERO));
        store.despawn(id).unwrap()
    }

    #[test]
    fn xp_curve_scales_with_level() {
        assert_eq!(xp_to_next(1), 100.0);
        assert_eq!(xp_to_next(2), 200.0);
        assert_eq!(xp_to_next(3), 300.0);
    }

    #[test]
    fn level_up_grants_point_restores_and_pauses() {
        let mut entity = player();
        entity.health.current = 10.0;
        if let Some(data) = entity.player_mut() {
            data.aura = 5.0;
        }
        let mut events = Vec::new();

        grant_xp(&mut entity, 100.0, &mut events);

        let data = entity.player().unwrap();
        assert_eq!(data.level, 2);
        assert_eq!(data.xp, 0.0);
        assert_eq!(data.points, 1);
        assert!(data.pending_level_up);
        assert_eq!(data.aura, 100.0);
        assert_eq!(entity.health.current, 100.0);
        assert!(events
            .iter()
            .any(|event| matches!(event, GameEvent::LevelUp { level: 2, .. })));
    }

    #[test]
    fn large_xp_grants_multiple_levels() {
        let mut entity = player();
        let mut events = Vec::new();

        grant_xp(&mut entity, 300.0, &mut events);

        let data = entity.player().unwrap();
        assert_eq!(data.level, 3);
        assert_eq!(data.points, 2);
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
    fn allocating_speed_raises_walk_speed() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        assert!(allocate_point(&mut entity, Attribute::Speed));
        assert_eq!(entity.stats.walk_speed, 230.0);
        assert_eq!(entity.player().unwrap().points, 0);
        assert!(!allocate_point(&mut entity, Attribute::Speed));
    }

    #[test]
    fn allocating_health_heals_ten() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        entity.health.current = 50.0;
        assert!(allocate_point(&mut entity, Attribute::Health));
        assert_eq!(entity.stats.max_health, 110.0);
        assert_eq!(entity.health.current, 60.0);
    }

    #[test]
    fn allocating_aura_raises_max_and_mogar_damage() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        assert!(allocate_point(&mut entity, Attribute::Aura));
        assert_eq!(entity.stats.max_aura, 110.0);
        assert_eq!(entity.stats.mogar_damage, 22.0);
    }

    #[test]
    fn allocating_strength_raises_melee_damage() {
        let mut entity = player();
        entity.player_mut().unwrap().points = 1;
        assert!(allocate_point(&mut entity, Attribute::Strength));
        assert_eq!(entity.stats.melee_damage(0), 8.0);
        assert_eq!(entity.stats.melee_damage(2), 14.0);
    }

    #[test]
    fn save_roundtrip_preserves_progress() {
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
        };

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
    }

    #[test]
    fn incompatible_save_is_ignored() {
        let mut entity = player();
        let save = SaveData {
            version: 999,
            level: 9,
            xp: 0.0,
            points: 9,
            attributes: Attributes::default(),
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
}
