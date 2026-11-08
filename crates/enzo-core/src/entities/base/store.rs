//! Armazenamento determinístico de entidades.
//!
//! `BTreeMap<EntityId, Entity>` garante iteração estável e IDs não reutilizados
//! no MVP. Times e relações de hostilidade também vivem aqui, para que os
//! sistemas consultem o mesmo contrato.

use std::collections::BTreeMap;

use crate::math::Rect;

use super::components::{Body, TeamId, TeamRelations};
use super::entity::{Entity, EntityId, EntityKind};
use crate::entities::actors::player::PlayerId;
use crate::entities::item::{EquipError, ItemData};
use crate::entities::spec::EntitySpec;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct EntityStore {
    next_id: u32,
    entities: BTreeMap<EntityId, Entity>,
    relations: TeamRelations,
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

        let (visual_character, visual_profile) = match &spec.kind {
            EntityKind::Player(data) => (Some(data.character), data.visual_profile),
            _ => (spec.visual_character, spec.visual_profile),
        };
        let entity = Entity {
            id,
            kind: spec.kind,
            controller: spec.controller,
            team: spec.team,
            body,
            health,
            stats: spec.stats,
            attributes: spec.attributes,
            equipment: spec.equipment,
            equipped: spec.equipped,
            locomotion: spec.locomotion,
            skills: spec.skills,
            visual_character,
            visual_profile,
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

    pub fn items(&self) -> impl Iterator<Item = &Entity> + '_ {
        self.entities.values().filter(|entity| entity.is_item())
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

    pub fn relations(&self) -> &TeamRelations {
        &self.relations
    }

    pub fn relations_mut(&mut self) -> &mut TeamRelations {
        &mut self.relations
    }

    pub fn in_rect(&self, rect: Rect, team: TeamId) -> Vec<EntityId> {
        self.entities
            .values()
            .filter(|entity| entity.team == team && entity.rect().overlaps(rect))
            .map(|entity| entity.id)
            .collect()
    }

    pub fn hostiles_in_rect(&self, rect: Rect, team: TeamId) -> Vec<EntityId> {
        self.entities
            .values()
            .filter(|entity| {
                self.relations.is_hostile(entity.team, team) && entity.rect().overlaps(rect)
            })
            .map(|entity| entity.id)
            .collect()
    }

    /// Equipa um item em uma entidade, validando tipo e atributos.
    pub fn equip(&mut self, id: EntityId, item: &ItemData) -> Result<(), EquipError> {
        let Some(entity) = self.get_mut(id) else {
            return Err(EquipError::NotEquippable);
        };
        entity.try_equip(item)
    }

    /// Substitui o `kind` sem recriar a entidade. Usado por save/load e testes.
    pub fn replace_kind(&mut self, id: EntityId, kind: EntityKind) -> bool {
        match self.entities.get_mut(&id) {
            Some(entity) => {
                entity.kind = kind;
                true
            }
            None => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entities::{
        AttributeId, Attributes, EquipSlot, EquipmentTags, ItemData, ENZO_SIZE, MOGGING_SPEED,
        SLIME_SIZE,
    };
    use crate::math::Rect;
    use crate::math::Vec2;

    fn store_with_actors() -> (EntityStore, EntityId, EntityId, EntityId) {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::new(0.0, 0.0)));
        let enemy = store.spawn(EntitySpec::slime(Vec2::new(100.0, 0.0)));
        let mogging = store.spawn(EntitySpec::mogging(
            player,
            Vec2::new(200.0, 10.0),
            1,
            20.0,
            TeamId::PLAYERS,
        ));
        (store, player, enemy, mogging)
    }

    #[test]
    fn player_and_slime_share_components() {
        let (store, player_id, enemy_id, _) = store_with_actors();
        let player = store.get(player_id).unwrap();
        let enemy = store.get(enemy_id).unwrap();

        assert_eq!(player.team, TeamId::PLAYERS);
        assert_eq!(enemy.team, TeamId::ENEMIES);
        assert_eq!(player.stats.max_health, 100.0);
        assert_eq!(enemy.stats.max_health, 30.0);
        assert_eq!(player.body.size, ENZO_SIZE);
        assert_eq!(enemy.body.size, SLIME_SIZE);
        assert_eq!(player.health.current, 100.0);
        assert_eq!(enemy.health.current, 30.0);
        assert!(player.is_actor());
        assert!(enemy.is_actor());
        assert!(player.is_playable());
        assert!(!enemy.is_playable());
    }

    #[test]
    fn ids_are_sequential_and_stable() {
        let (store, player, enemy, mogging) = store_with_actors();
        assert_eq!(player, EntityId(0));
        assert_eq!(enemy, EntityId(1));
        assert_eq!(mogging, EntityId(2));
        assert_eq!(store.player_entity_id(0), Some(player));
        assert_eq!(store.player(0).unwrap().id, player);
    }

    #[test]
    fn mogging_carries_owner_team_and_velocity() {
        let (store, player, _, mogging) = store_with_actors();
        let projectile = store.get(mogging).unwrap();
        let data = projectile.projectile().unwrap();
        assert_eq!(data.owner, player);
        assert_eq!(data.damage, 20.0);
        assert_eq!(data.skill, crate::skills::SkillId::Mogging);
        assert_eq!(projectile.body.vel.x, MOGGING_SPEED);
        assert_eq!(projectile.team, TeamId::PLAYERS);
        assert!(projectile.controller.is_none());
    }

    #[test]
    fn despawn_removes_entity() {
        let (mut store, _, enemy, _) = store_with_actors();
        assert!(store.despawn(enemy).is_some());
        assert!(store.get(enemy).is_none());
        assert!(store.despawn(enemy).is_none());
    }

    #[test]
    fn in_rect_filters_by_team() {
        let (store, player, enemy, _) = store_with_actors();
        let around_player = Rect::new(-10.0, -10.0, 50.0, 50.0);
        assert_eq!(store.in_rect(around_player, TeamId::PLAYERS), vec![player]);
        assert!(store.in_rect(around_player, TeamId::ENEMIES).is_empty());
        let around_enemy = Rect::new(90.0, -10.0, 40.0, 40.0);
        assert_eq!(store.in_rect(around_enemy, TeamId::ENEMIES), vec![enemy]);
    }

    #[test]
    fn hostiles_in_rect_uses_team_relations() {
        let (store, player, enemy, _) = store_with_actors();
        let all = Rect::new(-10.0, -10.0, 200.0, 50.0);
        assert_eq!(store.hostiles_in_rect(all, TeamId::PLAYERS), vec![enemy]);
        assert_eq!(store.hostiles_in_rect(all, TeamId::ENEMIES), vec![player]);
    }

    #[test]
    fn items_are_stored_but_not_actors() {
        let mut store = EntityStore::new();
        let item = store.spawn(EntitySpec::item(ItemData::SWORD, Vec2::new(10.0, 10.0)));
        assert_eq!(store.items().count(), 1);
        assert_eq!(store.actors().count(), 0);
        assert!(store.get(item).unwrap().is_item());
    }

    #[test]
    fn equip_validates_and_recomputes_aggregate() {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::ZERO));

        assert!(matches!(
            store.equip(player, &ItemData::SWORD),
            Err(EquipError::AttributesTooLow { .. })
        ));

        store.get_mut(player).unwrap().attributes.strength = 2;
        store.equip(player, &ItemData::SWORD).unwrap();

        let entity = store.get(player).unwrap();
        assert!(entity.equipment.has(EquipmentTags::SWORD));
        assert_eq!(entity.attribute_set().get(AttributeId::MELEE), 1);
        assert_eq!(entity.character_attribute_set().get(AttributeId::MELEE), 0);
    }

    #[test]
    fn equip_replaces_same_slot_and_keeps_other_slots() {
        let mut store = EntityStore::new();
        let player = store.spawn(EntitySpec::player(0, Vec2::ZERO));
        {
            let entity = store.get_mut(player).unwrap();
            entity.attributes.strength = 2;
            entity.attributes.health = 2;
        }

        store.equip(player, &ItemData::SWORD).unwrap();
        store.equip(player, &ItemData::ARMOR).unwrap();
        store.equip(player, &ItemData::BOW).unwrap_err(); // falta velocidade

        let entity = store.get(player).unwrap();
        assert!(entity.equipment.has(EquipmentTags::SWORD));
        assert_eq!(entity.equipment.modifiers.get(AttributeId::MELEE), 1);
        assert_eq!(entity.equipment.modifiers.get(AttributeId::HEALTH), 1);

        let removed = store
            .get_mut(player)
            .unwrap()
            .unequip(EquipSlot::Weapon)
            .unwrap();
        assert_eq!(removed.slot, EquipSlot::Weapon);
        let entity = store.get(player).unwrap();
        assert!(!entity.equipment.has(EquipmentTags::SWORD));
        assert_eq!(entity.equipment.modifiers.get(AttributeId::MELEE), 0);
    }

    #[test]
    fn enemy_cannot_equip_playable_item() {
        let mut store = EntityStore::new();
        let slime = store.spawn(EntitySpec::slime(Vec2::ZERO));
        store.get_mut(slime).unwrap().attributes = Attributes {
            strength: 99,
            ..Attributes::ZERO
        };
        assert!(matches!(
            store.equip(slime, &ItemData::SWORD),
            Err(EquipError::WrongEntityType { .. })
        ));
    }
}
