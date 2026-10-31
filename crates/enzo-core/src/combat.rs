//! Combate: combo físico de três golpes, Mogar, projéteis e dano de contato.

use crate::aura;
use crate::entity::{EntityId, EntitySpec, EntityStore, Faction, PlayerId, MOGAR_SIZE};
use crate::events::GameEvent;
use crate::map::Level;
use crate::math::{Rect, Vec2};

pub const MELEE_RANGE: f32 = 34.0;
pub const MELEE_HEIGHT: f32 = 26.0;
pub const MELEE_WINDOW: f32 = 0.5;
pub const MELEE_ANIM_TIME: f32 = 0.18;
pub const KNOCKBACK_SPEED: f32 = 220.0;
pub const KNOCKBACK_STUN: f32 = 0.18;
pub const MOGAR_COST: f32 = 25.0;
pub const MOGAR_COOLDOWN: f32 = 0.35;
pub const CONTACT_IFRAMES: f32 = 1.0;
pub const PLAYER_KNOCKBACK: f32 = 200.0;
pub const PLAYER_KNOCKBACK_LIFT: f32 = 220.0;

/// Avança o combo físico quando o jogador pressiona ataque neste tick.
pub fn melee(store: &mut EntityStore, attacker_id: EntityId, events: &mut Vec<GameEvent>) {
    let Some(attacker) = store.get_mut(attacker_id) else {
        return;
    };
    if !attacker.is_player() || !attacker.health.is_alive() {
        return;
    }
    if !attacker
        .player()
        .is_some_and(|player| player.input.melee_pressed)
    {
        return;
    }

    let step = if attacker.combat.melee_window > 0.0 {
        (attacker.combat.melee_step % 3) + 1
    } else {
        1
    };

    attacker.combat.melee_step = step;
    attacker.combat.melee_window = MELEE_WINDOW;
    attacker.combat.attack_timer = MELEE_ANIM_TIME;

    let facing = if attacker.body.facing >= 0 { 1.0 } else { -1.0 };
    let hitbox = if facing > 0.0 {
        Rect::new(
            attacker.body.right(),
            attacker.body.pos.y + (attacker.body.size.y - MELEE_HEIGHT) * 0.5,
            MELEE_RANGE,
            MELEE_HEIGHT,
        )
    } else {
        Rect::new(
            attacker.body.pos.x - MELEE_RANGE,
            attacker.body.pos.y + (attacker.body.size.y - MELEE_HEIGHT) * 0.5,
            MELEE_RANGE,
            MELEE_HEIGHT,
        )
    };

    let damage = attacker.stats.melee_damage((step - 1) as usize);
    let faction = attacker.faction;

    let targets: Vec<EntityId> = store
        .hostiles_in_rect(hitbox, faction)
        .into_iter()
        .filter(|id| *id != attacker_id)
        .filter(|id| store.get(*id).is_some_and(|entity| entity.is_actor()))
        .collect();

    for target_id in targets {
        let Some(target) = store.get_mut(target_id) else {
            continue;
        };
        let applied = target.health.damage(damage);
        if applied <= 0.0 {
            continue;
        }
        if let Some(enemy) = target.enemy_mut() {
            enemy.stun = KNOCKBACK_STUN;
        }
        if step == 3 {
            target.body.vel.x = facing * KNOCKBACK_SPEED;
        }
        events.push(GameEvent::MeleeHit {
            attacker: attacker_id,
            target: target_id,
            damage: applied,
            combo: step,
        });
    }
}

/// Dispara o Mogar se houver aura e o cooldown permitir.
pub fn fire_mogar(store: &mut EntityStore, player_id: PlayerId, events: &mut Vec<GameEvent>) {
    let Some(entity_id) = store.player_entity_id(player_id) else {
        return;
    };

    let fired = {
        let Some(player) = store.get_mut(entity_id) else {
            return;
        };
        if !player.health.is_alive() || !player.combat.can_fire_mogar() {
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

        player.combat.mogar_cooldown = MOGAR_COOLDOWN;
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

/// Dano de contato dos inimigos, respeitando i-frames do jogador.
pub fn contact_damage(store: &mut EntityStore, events: &mut Vec<GameEvent>) {
    let enemies: Vec<EntityId> = store.enemies().map(|entity| entity.id).collect();
    let players: Vec<EntityId> = store.players().map(|entity| entity.id).collect();

    for player_id in players {
        let Some(player) = store.get(player_id) else {
            continue;
        };
        if !player.health.is_alive() || player.health.iframes > 0.0 {
            continue;
        }
        let player_rect = player.rect();
        let player_center = player.center();

        for enemy_id in &enemies {
            let Some(enemy) = store.get(*enemy_id) else {
                continue;
            };
            if !enemy.health.is_alive() || !enemy.rect().overlaps(player_rect) {
                continue;
            }
            let damage = enemy.enemy().map(|data| data.contact_damage).unwrap_or(0.0);
            let direction = if player_center.x >= enemy.center().x {
                1.0
            } else {
                -1.0
            };

            let applied = store
                .get_mut(player_id)
                .map(|player| {
                    let applied = player.health.damage(damage);
                    player.health.iframes = CONTACT_IFRAMES;
                    player.body.vel.x = direction * PLAYER_KNOCKBACK;
                    player.body.vel.y = -PLAYER_KNOCKBACK_LIFT;
                    applied
                })
                .unwrap_or(0.0);

            if applied > 0.0 {
                events.push(GameEvent::PlayerDamaged {
                    player: player_id,
                    source: *enemy_id,
                    damage: applied,
                });
            }
            break;
        }
    }
}

/// Avança i-frames, janelas de combo, cooldowns e atordoamento.
pub fn tick_timers(store: &mut EntityStore, dt: f32) {
    for entity in store.iter_mut() {
        entity.health.tick(dt);
        entity.combat.tick(dt);
        if let Some(enemy) = entity.enemy_mut() {
            enemy.stun = (enemy.stun - dt).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::{Stats, PLAYER_SIZE};
    use crate::map::TILE_SIZE;

    fn level(source: &str) -> Level {
        Level::parse(source).unwrap()
    }

    fn setup() -> (EntityStore, EntityId, EntityId, Level) {
        let level = level("P...\n####\n");
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let enemy = store.spawn(EntitySpec::enemy(Vec2::new(30.0, 0.0)));
        (store, player, enemy, level)
    }

    fn press_melee(store: &mut EntityStore, player: EntityId, events: &mut Vec<GameEvent>) {
        if let Some(entity) = store.get_mut(player) {
            if let Some(data) = entity.player_mut() {
                data.input.melee_pressed = true;
            }
        }
        melee(store, player, events);
        if let Some(entity) = store.get_mut(player) {
            if let Some(data) = entity.player_mut() {
                data.input.melee_pressed = false;
            }
        }
    }

    #[test]
    fn three_hits_chain_and_third_knocks_back() {
        let (mut store, player, enemy, _) = setup();
        let mut events = Vec::new();

        press_melee(&mut store, player, &mut events);
        press_melee(&mut store, player, &mut events);
        press_melee(&mut store, player, &mut events);

        let target = store.get(enemy).unwrap();
        assert_eq!(target.health.current, 6.0);
        assert_eq!(target.body.vel.x, KNOCKBACK_SPEED);
        assert_eq!(
            events
                .iter()
                .filter(|event| matches!(event, GameEvent::MeleeHit { .. }))
                .count(),
            3
        );
        assert!(matches!(
            events.last(),
            Some(GameEvent::MeleeHit { combo: 3, .. })
        ));
    }

    #[test]
    fn combo_resets_after_window() {
        let (mut store, player, _, _) = setup();
        let mut events = Vec::new();

        press_melee(&mut store, player, &mut events);
        press_melee(&mut store, player, &mut events);
        tick_timers(&mut store, MELEE_WINDOW + crate::DT);
        press_melee(&mut store, player, &mut events);

        let attacker = store.get(player).unwrap();
        assert_eq!(attacker.combat.melee_step, 1);
        assert!(matches!(
            events.last(),
            Some(GameEvent::MeleeHit { combo: 1, .. })
        ));
    }

    #[test]
    fn melee_does_not_hit_behind() {
        let (mut store, player, enemy, _) = setup();
        let mut events = Vec::new();
        store.get_mut(player).unwrap().body.facing = -1;

        press_melee(&mut store, player, &mut events);

        assert_eq!(store.get(enemy).unwrap().health.current, 30.0);
        assert!(events.is_empty());
    }

    #[test]
    fn mogar_costs_aura_and_spawns_projectile() {
        let (mut store, player, _, _) = setup();
        let mut events = Vec::new();
        if let Some(data) = store.get_mut(player).unwrap().player_mut() {
            data.input.mogar_pressed = true;
        }

        fire_mogar(&mut store, 0, &mut events);

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
        let (mut store, player, _, _) = setup();
        let mut events = Vec::new();
        if let Some(data) = store.get_mut(player).unwrap().player_mut() {
            data.aura = 10.0;
            data.input.mogar_pressed = true;
        }

        fire_mogar(&mut store, 0, &mut events);

        assert_eq!(
            store.iter().filter(|entity| entity.is_projectile()).count(),
            0
        );
        assert!(events
            .iter()
            .any(|event| matches!(event, GameEvent::MogarNoAura { .. })));
    }

    #[test]
    fn mogar_hits_each_enemy_once_and_passes_through() {
        let level = level("P.......\n########\n");
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let first = store.spawn(EntitySpec::enemy(Vec2::new(60.0, 0.0)));
        let second = store.spawn(EntitySpec::enemy(Vec2::new(120.0, 0.0)));
        let mut events = Vec::new();
        if let Some(data) = store.get_mut(player).unwrap().player_mut() {
            data.input.mogar_pressed = true;
        }
        fire_mogar(&mut store, 0, &mut events);

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
        if let Some(data) = store.get_mut(player).unwrap().player_mut() {
            data.input.mogar_pressed = true;
        }
        fire_mogar(&mut store, 0, &mut events);

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

    #[test]
    fn contact_damage_respects_iframes() {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        store.spawn(EntitySpec::enemy(Vec2::new(0.0, 0.0)));
        let mut events = Vec::new();

        contact_damage(&mut store, &mut events);
        assert_eq!(store.get(player).unwrap().health.current, 90.0);
        assert_eq!(store.get(player).unwrap().health.iframes, CONTACT_IFRAMES);
        assert!(events
            .iter()
            .any(|event| matches!(event, GameEvent::PlayerDamaged { .. })));

        contact_damage(&mut store, &mut events);
        assert_eq!(store.get(player).unwrap().health.current, 90.0);

        tick_timers(&mut store, CONTACT_IFRAMES + crate::DT);
        contact_damage(&mut store, &mut events);
        assert_eq!(store.get(player).unwrap().health.current, 80.0);
    }

    #[test]
    fn stats_and_sizes_are_shared_between_actors() {
        let (store, player, enemy, _) = setup();
        let player_entity = store.get(player).unwrap();
        let enemy_entity = store.get(enemy).unwrap();
        assert_eq!(player_entity.body.size, PLAYER_SIZE);
        assert_eq!(player_entity.stats, Stats::player(Default::default()));
        assert_eq!(enemy_entity.stats, Stats::enemy());
        assert_eq!(player_entity.health.current, 100.0);
        assert_eq!(enemy_entity.health.current, 30.0);
        assert_eq!(TILE_SIZE, 32.0);
    }
}
