//! IA da slime: patrulha simples.
//!
//! A slime anda em linha reta e vira ao encontrar parede ou beirada. Não pula,
//! não persegue e não ataca à distância; o dano é por contato.

use crate::entities::Entity;
use crate::map::Level;
use crate::math::Vec2;

/// Estado próprio da slime. Fica dentro de [`crate::ai::AiBehavior::Slime`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SlimeAi {
    pub patrol_dir: i8,
}

impl SlimeAi {
    pub const fn new(patrol_dir: i8) -> Self {
        Self { patrol_dir }
    }
}

impl Default for SlimeAi {
    fn default() -> Self {
        Self { patrol_dir: -1 }
    }
}

/// Calcula a intenção do tick e devolve o estado atualizado.
pub fn update(entity: &mut Entity, level: &Level, state: SlimeAi) -> SlimeAi {
    let Some(enemy) = entity.enemy().copied() else {
        entity.locomotion.walk.intent_x = 0;
        return state;
    };

    if enemy.stun > 0.0 {
        entity.locomotion.walk.intent_x = 0;
        return state;
    }

    let mut direction = if state.patrol_dir == 0 {
        -1
    } else {
        state.patrol_dir
    };

    let ahead_x = if direction > 0 {
        entity.body.right() + 1.0
    } else {
        entity.body.pos.x - 1.0
    };
    let center = Vec2::new(ahead_x, entity.body.center().y);
    let below = Vec2::new(ahead_x, entity.body.bottom() + 2.0);

    let wall = level.solid_at_pos(center);
    let ledge = entity.body.on_ground && !level.standable_at_pos(below);

    if wall || ledge {
        direction = -direction;
    }

    entity.locomotion.walk.intent_x = direction;
    entity.body.facing = direction;

    SlimeAi {
        patrol_dir: direction,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::Ai;
    use crate::entities::{EntityId, EntitySpec, EntityStore};

    fn level(source: &str) -> Level {
        Level::parse(source).unwrap()
    }

    fn slime_at(store: &mut EntityStore, x: f32, y: f32, dir: i8) -> EntityId {
        let id = store.spawn(EntitySpec::slime(Vec2::new(x, y)));
        let entity = store.get_mut(id).unwrap();
        *entity.ai_mut().unwrap() = Ai::slime_with_dir(dir);
        id
    }

    fn patrol_dir(store: &EntityStore, id: EntityId) -> i8 {
        store
            .get(id)
            .and_then(|entity| entity.ai().copied())
            .map(Ai::patrol_dir)
            .unwrap_or(0)
    }

    #[test]
    fn slime_turns_at_wall() {
        let level = level("P.#\n###\n");
        let mut store = EntityStore::new();
        // +1,+1 shift from surrounding barrier.
        let enemy = slime_at(&mut store, 64.0, 38.0, 1);
        store.get_mut(enemy).unwrap().body.on_ground = true;

        // Anda para a direita; parede no tile 3 (x >= 96).
        crate::ai::update(&mut store, &level);
        assert_eq!(store.get(enemy).unwrap().locomotion.walk.intent_x, 1);

        store.get_mut(enemy).unwrap().body.pos.x = 90.0;
        crate::ai::update(&mut store, &level);
        assert_eq!(store.get(enemy).unwrap().locomotion.walk.intent_x, -1);
        assert_eq!(patrol_dir(&store, enemy), -1);
    }

    #[test]
    fn slime_turns_at_ledge() {
        let level = level("P..\n##.\n");
        let mut store = EntityStore::new();
        let enemy = slime_at(&mut store, 40.0, 6.0, 1);
        store.get_mut(enemy).unwrap().body.on_ground = true;

        crate::ai::update(&mut store, &level);
        assert_eq!(store.get(enemy).unwrap().locomotion.walk.intent_x, -1);
    }

    #[test]
    fn stunned_slime_stops_and_keeps_state() {
        let level = level("P...\n####\n");
        let mut store = EntityStore::new();
        let enemy = slime_at(&mut store, 32.0, 6.0, 1);
        if let Some(data) = store.get_mut(enemy).unwrap().enemy_mut() {
            data.stun = 0.2;
        }

        crate::ai::update(&mut store, &level);
        assert_eq!(store.get(enemy).unwrap().locomotion.walk.intent_x, 0);
        assert_eq!(patrol_dir(&store, enemy), 1);
    }
}
