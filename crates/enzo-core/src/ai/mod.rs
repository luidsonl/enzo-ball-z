//! IA componentizada.
//!
//! Cada entidade com IA carrega um componente [`Ai`] com o comportamento e o
//! estado daquele comportamento. O despachante [`update`] só traduz estado em
//! intenção (`locomotion.walk.intent_x`); física continua em `mechanics`/`physics`.
//!
//! Para adicionar um inimigo novo: crie `ai/<especie>.rs`, adicione uma variante
//! em [`AiBehavior`], um construtor em [`Ai`] e ligue no `EntitySpec`.

pub mod slime;

pub use slime::SlimeAi;

use crate::entities::{EntityId, EntityStore};
use crate::map::Level;

/// Comportamentos conhecidos. Cada variante carrega o estado do comportamento,
/// então dois inimigos da mesma espécie não compartilham memória.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AiBehavior {
    Slime(SlimeAi),
}

impl AiBehavior {
    /// Direção de patrulha para o snapshot. Comportamentos sem patrulha usam 0.
    pub fn patrol_dir(self) -> i8 {
        match self {
            Self::Slime(state) => state.patrol_dir,
        }
    }
}

/// Componente de IA anexado a entidades não-jogáveis.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ai {
    pub behavior: AiBehavior,
    pub enabled: bool,
}

impl Ai {
    pub const fn new(behavior: AiBehavior) -> Self {
        Self {
            behavior,
            enabled: true,
        }
    }

    pub fn slime() -> Self {
        Self::new(AiBehavior::Slime(SlimeAi::default()))
    }

    pub fn slime_with_dir(patrol_dir: i8) -> Self {
        Self::new(AiBehavior::Slime(SlimeAi::new(patrol_dir)))
    }

    pub fn patrol_dir(self) -> i8 {
        self.behavior.patrol_dir()
    }
}

/// Avança a IA de todas as entidades que possuem o componente.
pub fn update(store: &mut EntityStore, level: &Level) {
    let ids: Vec<EntityId> = store
        .iter()
        .filter(|entity| entity.controller.is_ai())
        .map(|entity| entity.id)
        .collect();

    for id in ids {
        let Some(entity) = store.get_mut(id) else {
            continue;
        };
        let Some(mut ai) = entity.ai().copied() else {
            continue;
        };

        if !ai.enabled || !entity.health.is_alive() {
            entity.locomotion.walk.intent_x = 0;
            continue;
        }

        ai.behavior = match ai.behavior {
            AiBehavior::Slime(state) => AiBehavior::Slime(slime::update(entity, level, state)),
        };
        if let Some(slot) = entity.ai_mut() {
            *slot = ai;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::EntitySpec;
    use crate::math::Vec2;

    fn level(source: &str) -> Level {
        Level::parse(source).unwrap()
    }

    #[test]
    fn slime_behavior_is_attached_by_spec() {
        let spec = EntitySpec::slime(Vec2::ZERO);
        let ai = spec.controller.ai().copied().expect("slime deve ter IA");
        assert!(ai.enabled);
        assert_eq!(ai.behavior, AiBehavior::Slime(SlimeAi::new(-1)));
    }

    #[test]
    fn disabled_ai_stops_intent() {
        let level = level("P...\n####\n");
        let mut store = EntityStore::new();
        let enemy = store.spawn(EntitySpec::slime(Vec2::new(32.0, 6.0)));
        {
            let entity = store.get_mut(enemy).unwrap();
            entity.locomotion.walk.intent_x = 1;
            if let Some(ai) = entity.ai_mut() {
                ai.enabled = false;
            }
        }

        update(&mut store, &level);

        assert_eq!(store.get(enemy).unwrap().locomotion.walk.intent_x, 0);
    }
}
