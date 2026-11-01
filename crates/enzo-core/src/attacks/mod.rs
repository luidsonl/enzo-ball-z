//! Movimentos de ataque.
//!
//! Cada ataque vive em seu próprio módulo (`melee`, `mogar`) e declara seus
//! `REQUIREMENTS`. [`Combat`] reúne os componentes e o [`MoveSet`] atribuído à
//! entidade.
//!
//! Novos golpes entram como um módulo novo, uma variante de `MoveId` e uma etapa
//! no `World`. Nunca como lógica solta no `World` ou no cliente.

pub mod melee;
pub mod mogar;

pub use melee::MeleeCombo;
pub use mogar::MogarAbility;

use crate::entities::{EntityId, EntityStore};
use crate::events::GameEvent;
use crate::moves::{AbilityContext, AssignError, MoveCategory, MoveId, MoveSet};

pub const CONTACT_IFRAMES: f32 = 1.0;
pub const PLAYER_KNOCKBACK: f32 = 200.0;
pub const PLAYER_KNOCKBACK_LIFT: f32 = 220.0;

/// Componente de combate. Cada movimento de ataque tem seu próprio estado;
/// novos especiais entram como novos componentes aqui.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Combat {
    pub moves: MoveSet,
    pub melee: MeleeCombo,
    pub mogar: MogarAbility,
}

impl Default for Combat {
    fn default() -> Self {
        Self::none()
    }
}

impl Combat {
    pub fn none() -> Self {
        Self {
            moves: MoveSet::new(),
            melee: MeleeCombo::default(),
            mogar: MogarAbility::default(),
        }
    }

    pub fn player() -> Self {
        Self::from_moves(&[MoveId::Melee, MoveId::Mogar])
    }

    pub(crate) fn from_moves(ids: &[MoveId]) -> Self {
        let mut combat = Self::none();
        combat.moves = MoveSet::from_slice(ids);
        combat
    }

    pub fn has(&self, id: MoveId) -> bool {
        self.moves.contains(id)
    }

    /// Atribui um ataque se ele for da categoria correta e a entidade cumprir os
    /// requisitos declarados pelo ataque.
    pub fn try_assign(&mut self, id: MoveId, context: AbilityContext) -> Result<(), AssignError> {
        if id.category() != MoveCategory::Attack {
            return Err(AssignError::WrongCategory);
        }
        self.moves.try_assign(id, context)
    }

    pub fn with_move(mut self, id: MoveId, context: AbilityContext) -> Result<Self, AssignError> {
        self.try_assign(id, context)?;
        Ok(self)
    }

    pub fn tick(&mut self, dt: f32) {
        if self.has(MoveId::Melee) {
            self.melee.tick(dt);
        }
        if self.has(MoveId::Mogar) {
            self.mogar.tick(dt);
        }
    }

    pub fn can_fire_mogar(self) -> bool {
        self.has(MoveId::Mogar) && self.mogar.cooldown <= 0.0
    }
}

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
    use crate::entities::{EntitySpec, Stats, ENZO_SIZE};
    use crate::map::TILE_SIZE;
    use crate::math::Vec2;
    use crate::moves::EntityType;

    fn setup() -> (EntityStore, EntityId, EntityId) {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let enemy = store.spawn(EntitySpec::slime(Vec2::new(0.0, 0.0)));
        (store, player, enemy)
    }

    #[test]
    fn player_has_attacks_and_enemy_has_none() {
        let player = Combat::player();
        assert!(player.has(MoveId::Melee));
        assert!(player.has(MoveId::Mogar));
        assert!(!player.has(MoveId::Walk));

        let enemy = Combat::none();
        assert!(!enemy.has(MoveId::Melee));
        assert!(!enemy.has(MoveId::Mogar));
    }

    #[test]
    fn combat_rejects_movement_moves() {
        let mut combat = Combat::player();
        assert_eq!(
            combat.try_assign(MoveId::Jump, AbilityContext::player(1, Default::default())),
            Err(AssignError::WrongCategory)
        );
    }

    #[test]
    fn enemy_cannot_receive_mogar() {
        let mut combat = Combat::none();
        assert!(matches!(
            combat.try_assign(MoveId::Mogar, AbilityContext::enemy()),
            Err(AssignError::WrongCategory | AssignError::WrongEntityType { .. })
        ));
        assert!(!combat.has(MoveId::Mogar));
    }

    #[test]
    fn enemy_cannot_receive_player_only_attack() {
        let mut combat = Combat::none();
        assert!(matches!(
            combat.try_assign(MoveId::Melee, AbilityContext::enemy()),
            Err(AssignError::WrongEntityType {
                current: EntityType::Enemy,
                ..
            })
        ));
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

    #[test]
    fn stats_and_sizes_are_shared_between_actors() {
        let (store, player, enemy) = setup();
        let player_entity = store.get(player).unwrap();
        let enemy_entity = store.get(enemy).unwrap();
        assert_eq!(player_entity.body.size, ENZO_SIZE);
        assert_eq!(player_entity.stats, Stats::player(Default::default()));
        assert_eq!(enemy_entity.stats, Stats::slime());
        assert_eq!(player_entity.health.current, 100.0);
        assert_eq!(enemy_entity.health.current, 30.0);
        assert_eq!(TILE_SIZE, 32.0);
    }
}
