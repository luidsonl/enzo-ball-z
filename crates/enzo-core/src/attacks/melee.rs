//! Combo físico de três golpes.

use crate::entity::{Attributes, EntityId, EntityStore};
use crate::events::GameEvent;
use crate::math::Rect;
use crate::moves::{EntityType, MoveId, Requirements};

pub const MELEE_RANGE: f32 = 34.0;
pub const MELEE_HEIGHT: f32 = 26.0;
pub const MELEE_WINDOW: f32 = 0.5;
pub const MELEE_ANIM_TIME: f32 = 0.18;
pub const KNOCKBACK_SPEED: f32 = 220.0;
pub const KNOCKBACK_STUN: f32 = 0.18;

/// O combo físico é exclusivo de jogadores no MVP.
pub const REQUIREMENTS: Requirements =
    Requirements::new(&[EntityType::Player], 1, Attributes::ZERO);

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct MeleeCombo {
    pub step: u8,
    pub window: f32,
    pub anim_timer: f32,
}

impl MeleeCombo {
    pub fn tick(&mut self, dt: f32) {
        self.window = (self.window - dt).max(0.0);
        if self.window <= 0.0 {
            self.step = 0;
        }
        self.anim_timer = (self.anim_timer - dt).max(0.0);
    }
}

/// Avança o combo quando o jogador pressiona ataque neste tick.
pub fn update(store: &mut EntityStore, attacker_id: EntityId, events: &mut Vec<GameEvent>) {
    let Some(attacker) = store.get_mut(attacker_id) else {
        return;
    };
    if !attacker.health.is_alive() || !attacker.combat.has(MoveId::Melee) {
        return;
    }
    if !attacker
        .player()
        .is_some_and(|player| player.input.melee_pressed)
    {
        return;
    }

    let step = if attacker.combat.melee.window > 0.0 {
        (attacker.combat.melee.step % 3) + 1
    } else {
        1
    };

    attacker.combat.melee.step = step;
    attacker.combat.melee.window = MELEE_WINDOW;
    attacker.combat.melee.anim_timer = MELEE_ANIM_TIME;

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::EntitySpec;
    use crate::math::Vec2;

    fn setup() -> (EntityStore, EntityId, EntityId) {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let enemy = store.spawn(EntitySpec::enemy(Vec2::new(30.0, 0.0)));
        (store, player, enemy)
    }

    fn press_melee(store: &mut EntityStore, player: EntityId, events: &mut Vec<GameEvent>) {
        if let Some(entity) = store.get_mut(player) {
            if let Some(data) = entity.player_mut() {
                data.input.melee_pressed = true;
            }
        }
        update(store, player, events);
        if let Some(entity) = store.get_mut(player) {
            if let Some(data) = entity.player_mut() {
                data.input.melee_pressed = false;
            }
        }
    }

    #[test]
    fn three_hits_chain_and_third_knocks_back() {
        let (mut store, player, enemy) = setup();
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
        let (mut store, player, _) = setup();
        let mut events = Vec::new();

        press_melee(&mut store, player, &mut events);
        press_melee(&mut store, player, &mut events);
        store
            .get_mut(player)
            .unwrap()
            .combat
            .tick(MELEE_WINDOW + crate::DT);
        press_melee(&mut store, player, &mut events);

        let attacker = store.get(player).unwrap();
        assert_eq!(attacker.combat.melee.step, 1);
        assert!(matches!(
            events.last(),
            Some(GameEvent::MeleeHit { combo: 1, .. })
        ));
    }

    #[test]
    fn melee_does_not_hit_behind() {
        let (mut store, player, enemy) = setup();
        let mut events = Vec::new();
        store.get_mut(player).unwrap().body.facing = -1;

        press_melee(&mut store, player, &mut events);

        assert_eq!(store.get(enemy).unwrap().health.current, 30.0);
        assert!(events.is_empty());
    }

    #[test]
    fn entity_without_melee_move_cannot_attack() {
        let (mut store, player, enemy) = setup();
        store
            .get_mut(player)
            .unwrap()
            .combat
            .moves
            .remove(MoveId::Melee);
        let mut events = Vec::new();

        press_melee(&mut store, player, &mut events);

        assert_eq!(store.get(enemy).unwrap().health.current, 30.0);
        assert!(events.is_empty());
    }
}
