//! Mogar: projétil que gasta aura e atravessa inimigos.

use crate::aura;
use crate::entity::{Attributes, EntityId, EntitySpec, EntityStore, Faction, PlayerId, MOGAR_SIZE};
use crate::events::GameEvent;
use crate::map::Level;
use crate::math::{Rect, Vec2};
use crate::moves::{EntityType, MoveId, Requirements};

pub const MOGAR_COST: f32 = 25.0;
pub const MOGAR_COOLDOWN: f32 = 0.35;

/// Mogar é exclusivo de jogadores no MVP.
pub const REQUIREMENTS: Requirements =
    Requirements::new(&[EntityType::Player], 1, Attributes::ZERO);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MogarAbility {
    pub cooldown: f32,
}

impl MogarAbility {
    pub fn tick(&mut self, dt: f32) {
        self.cooldown = (self.cooldown - dt).max(0.0);
    }
}

/// Dispara o Mogar se houver aura, movimento atribuído e cooldown livre.
pub fn fire(store: &mut EntityStore, player_id: PlayerId, events: &mut Vec<GameEvent>) {
    let Some(entity_id) = store.player_entity_id(player_id) else {
        return;
    };

    let fired = {
        let Some(player) = store.get_mut(entity_id) else {
            return;
        };
        if !player.health.is_alive()
            || !player.combat.has(MoveId::Mogar)
            || !player.combat.can_fire_mogar()
        {
            return;
        }
        if !player.player().is_some_and(|data| data.input.mogar_pressed) {
            return;
        }

        let Some(data) = player.player_mut() else {
            return;
        };
        if !aura::spend(data, MOGAR_COST) {
            events.push(GameEvent::MogarNoAura { owner: entity_id });
            return;
        }

        player.combat.mogar.cooldown = MOGAR_COOLDOWN;
        let facing = if player.body.facing >= 0 { 1 } else { -1 };
        let pos = Vec2::new(
            if facing > 0 {
                player.body.right() + 4.0
            } else {
                player.body.pos.x - MOGAR_SIZE.x - 4.0
            },
            player.body.pos.y + (player.body.size.y - MOGAR_SIZE.y) * 0.5,
        );
        Some((pos, facing, player.stats.mogar_damage))
    };

    let Some((pos, facing, damage)) = fired else {
        return;
    };

    let projectile = store.spawn(EntitySpec::mogar(entity_id, pos, facing, damage));
    events.push(GameEvent::MogarFired {
        owner: entity_id,
        projectile,
    });
}

/// Move projéteis, aplica dano e remove os que expiraram ou bateram em parede.
pub fn update_projectiles(
    store: &mut EntityStore,
    level: &Level,
    dt: f32,
    events: &mut Vec<GameEvent>,
) {
    let ids: Vec<EntityId> = store
        .iter()
        .filter(|entity| entity.is_projectile())
        .map(|entity| entity.id)
        .collect();

    for id in ids {
        let Some(entity) = store.get(id) else {
            continue;
        };
        let Some(data) = entity.projectile() else {
            continue;
        };
        let owner = data.owner;
        let damage = data.damage;
        let hits = data.hits.clone();
        let lifetime = data.lifetime;
        let pos = entity.body.pos;
        let vel = entity.body.vel;
        let size = entity.body.size;

        let new_pos = pos + vel * dt;
        let rect = Rect::new(new_pos.x, new_pos.y, size.x, size.y);

        if level.rect_overlaps_solid(rect) {
            store.despawn(id);
            events.push(GameEvent::MogarBlocked { owner });
            continue;
        }

        if let Some(entity) = store.get_mut(id) {
            entity.body.pos = new_pos;
            if let Some(data) = entity.projectile_mut() {
                data.lifetime = lifetime - dt;
            }
        }

        let targets: Vec<EntityId> = store
            .iter()
            .filter(|entity| entity.id != id)
            .filter(|entity| entity.is_actor())
            .filter(|entity| entity.faction.hostile_to(Faction::Players))
            .filter(|entity| entity.rect().overlaps(rect))
            .filter(|entity| !hits.contains(&entity.id))
            .map(|entity| entity.id)
            .collect();

        for target_id in targets {
            let applied = store
                .get_mut(target_id)
                .map(|target| target.health.damage(damage))
                .unwrap_or(0.0);
            if applied <= 0.0 {
                continue;
            }
            if let Some(data) = store.get_mut(id).and_then(|entity| entity.projectile_mut()) {
                data.hits.push(target_id);
            }
            events.push(GameEvent::MogarHit {
                owner,
                target: target_id,
                damage: applied,
            });
        }

        let expired = store
            .get(id)
            .and_then(|entity| entity.projectile())
            .is_some_and(|data| data.lifetime <= 0.0);
        if expired {
            store.despawn(id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EntitySpec;
    use crate::math::Vec2;

    fn level(source: &str) -> Level {
        Level::parse(source).unwrap()
    }

    fn setup() -> (EntityStore, EntityId) {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        (store, player)
    }

    fn press_mogar(store: &mut EntityStore, player: EntityId) {
        if let Some(data) = store.get_mut(player).unwrap().player_mut() {
            data.input.mogar_pressed = true;
        }
    }

    #[test]
    fn mogar_costs_aura_and_spawns_projectile() {
        let (mut store, player) = setup();
        let mut events = Vec::new();
        press_mogar(&mut store, player);

        fire(&mut store, 0, &mut events);

        assert_eq!(store.get(player).unwrap().aura(), 75.0);
        assert_eq!(
            store.iter().filter(|entity| entity.is_projectile()).count(),
            1
        );
        assert!(events
            .iter()
            .any(|event| matches!(event, GameEvent::MogarFired { .. })));
    }

    #[test]
    fn mogar_without_aura_does_not_fire() {
        let (mut store, player) = setup();
        let mut events = Vec::new();
        if let Some(data) = store.get_mut(player).unwrap().player_mut() {
            data.aura = 10.0;
            data.input.mogar_pressed = true;
        }

        fire(&mut store, 0, &mut events);

        assert_eq!(
            store.iter().filter(|entity| entity.is_projectile()).count(),
            0
        );
        assert!(events
            .iter()
            .any(|event| matches!(event, GameEvent::MogarNoAura { .. })));
    }

    #[test]
    fn mogar_requires_assigned_move() {
        let (mut store, player) = setup();
        store
            .get_mut(player)
            .unwrap()
            .combat
            .moves
            .remove(MoveId::Mogar);
        let mut events = Vec::new();
        press_mogar(&mut store, player);

        fire(&mut store, 0, &mut events);

        assert_eq!(store.get(player).unwrap().aura(), 100.0);
        assert!(events.is_empty());
    }

    #[test]
    fn mogar_hits_each_enemy_once_and_passes_through() {
        let level = level("P.......\n########\n");
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let first = store.spawn(EntitySpec::enemy(Vec2::new(60.0, 0.0)));
        let second = store.spawn(EntitySpec::enemy(Vec2::new(120.0, 0.0)));
        let mut events = Vec::new();
        press_mogar(&mut store, player);
        fire(&mut store, 0, &mut events);

        for _ in 0..100 {
            update_projectiles(&mut store, &level, crate::DT, &mut events);
        }

        assert_eq!(store.get(first).unwrap().health.current, 10.0);
        assert_eq!(store.get(second).unwrap().health.current, 10.0);
        assert_eq!(
            store.iter().filter(|entity| entity.is_projectile()).count(),
            0
        );
    }

    #[test]
    fn mogar_disappears_on_solid_tile() {
        let level = level("P..#\n####\n");
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let mut events = Vec::new();
        press_mogar(&mut store, player);
        fire(&mut store, 0, &mut events);

        for _ in 0..20 {
            update_projectiles(&mut store, &level, crate::DT, &mut events);
        }

        assert_eq!(
            store.iter().filter(|entity| entity.is_projectile()).count(),
            0
        );
        assert!(events
            .iter()
            .any(|event| matches!(event, GameEvent::MogarBlocked { .. })));
    }
}
