//! Dano de contato e timers comuns de combate.
//!
//! As skills de ataque vivem em [`crate::skills`]; aqui fica apenas o que não é
//! de nenhum slot: dano por encostar em inimigo, i-frames e atordoamento.

use crate::entities::{EntityId, EntityStore};
use crate::events::GameEvent;

pub const CONTACT_IFRAMES: f32 = 1.0;
pub const PLAYER_KNOCKBACK: f32 = 200.0;
pub const PLAYER_KNOCKBACK_LIFT: f32 = 220.0;

/// Dano de contato dos inimigos, respeitando i-frames do jogador e o time.
pub fn contact_damage(store: &mut EntityStore, events: &mut Vec<GameEvent>) {
    let players: Vec<EntityId> = store.players().map(|entity| entity.id).collect();

    for player_id in players {
        let Some((player_rect, player_center, player_team)) = store
            .get(player_id)
            .filter(|player| player.health.is_alive() && player.health.iframes <= 0.0)
            .map(|player| (player.rect(), player.center(), player.team))
        else {
            continue;
        };

        let hostiles = store.hostiles_in_rect(player_rect, player_team);
        for enemy_id in hostiles {
            let Some(enemy) = store.get(enemy_id) else {
                continue;
            };
            if !enemy.is_enemy() || !enemy.health.is_alive() {
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
                    source: enemy_id,
                    damage: applied,
                });
            }
            break;
        }
    }
}

/// Avança i-frames e atordoamento. Timers de skill ficam em
/// [`crate::skills::Skills::tick`].
pub fn tick_timers(store: &mut EntityStore, dt: f32) {
    for entity in store.iter_mut() {
        entity.health.tick(dt);
        if let Some(enemy) = entity.enemy_mut() {
            enemy.stun = (enemy.stun - dt).max(0.0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{EntityId, EntitySpec};
    use crate::math::Vec2;

    fn setup() -> (EntityStore, EntityId, EntityId) {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let enemy = store.spawn(EntitySpec::slime(Vec2::new(0.0, 0.0)));
        (store, player, enemy)
    }

    #[test]
    fn contact_damage_respects_iframes() {
        let (mut store, player, _) = setup();
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
}
