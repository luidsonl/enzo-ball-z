//! Entidade componentizada.
//!
//! Jogável, inimigo e projétil são o mesmo [`Entity`]. O que muda é o
//! [`EntityKind`] (payload específico) e o [`Controller`] (quem decide a
//! intenção). Componentes comuns (`Body`, `Health`, `Stats`, `Attributes`,
//! `TeamId`, `Movement`, `Combat`) existem em todas as entidades; `Equipment` e
//! os itens equipados são opcionais.
//!
//! Toda criação passa por [`crate::entities::EntityStore::spawn`] com um
//! [`crate::entities::EntitySpec`].

use crate::ai::Ai;
use crate::attacks::Combat;
use crate::math::{Rect, Vec2};
use crate::movements::Movement;
use crate::moves::{AbilityContext, AssignError, EntityType, MoveCategory, MoveId};

use super::attributes::{Attributes, Stats};
use super::components::{Body, Health, TeamId};
use super::controller::Controller;
use super::enemy::EnemyData;
use super::equipment::{EquipSlot, Equipment, EquippedItem};
use super::item::{EquipError, ItemData};
use super::player::PlayerData;
use super::projectile::ProjectileData;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityId(pub u32);

/// Payload específico de cada tipo de entidade. Os componentes comuns ficam no
/// [`Entity`], nunca duplicados aqui.
#[derive(Clone, Debug, PartialEq)]
pub enum EntityKind {
    Player(PlayerData),
    Enemy(EnemyData),
    Projectile(ProjectileData),
    Item(ItemData),
}

impl EntityKind {
    pub fn is_player(&self) -> bool {
        matches!(self, Self::Player(_))
    }

    pub fn is_enemy(&self) -> bool {
        matches!(self, Self::Enemy(_))
    }

    pub fn is_projectile(&self) -> bool {
        matches!(self, Self::Projectile(_))
    }

    pub fn is_item(&self) -> bool {
        matches!(self, Self::Item(_))
    }

    pub fn player(&self) -> Option<&PlayerData> {
        match self {
            Self::Player(data) => Some(data),
            _ => None,
        }
    }

    pub fn player_mut(&mut self) -> Option<&mut PlayerData> {
        match self {
            Self::Player(data) => Some(data),
            _ => None,
        }
    }

    pub fn enemy(&self) -> Option<&EnemyData> {
        match self {
            Self::Enemy(data) => Some(data),
            _ => None,
        }
    }

    pub fn enemy_mut(&mut self) -> Option<&mut EnemyData> {
        match self {
            Self::Enemy(data) => Some(data),
            _ => None,
        }
    }

    pub fn projectile(&self) -> Option<&ProjectileData> {
        match self {
            Self::Projectile(data) => Some(data),
            _ => None,
        }
    }

    pub fn projectile_mut(&mut self) -> Option<&mut ProjectileData> {
        match self {
            Self::Projectile(data) => Some(data),
            _ => None,
        }
    }

    pub fn item(&self) -> Option<&ItemData> {
        match self {
            Self::Item(data) => Some(data),
            _ => None,
        }
    }

    pub fn item_mut(&mut self) -> Option<&mut ItemData> {
        match self {
            Self::Item(data) => Some(data),
            _ => None,
        }
    }

    pub fn entity_type(&self) -> EntityType {
        match self {
            Self::Player(_) => EntityType::Playable,
            Self::Enemy(_) => EntityType::Enemy,
            Self::Projectile(_) => EntityType::Projectile,
            Self::Item(_) => EntityType::Item,
        }
    }

    pub fn level(&self) -> u32 {
        match self {
            Self::Player(data) => data.level,
            Self::Enemy(_) | Self::Projectile(_) | Self::Item(_) => 1,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entity {
    pub id: EntityId,
    pub kind: EntityKind,
    pub controller: Controller,
    pub team: TeamId,
    pub body: Body,
    pub health: Health,
    pub stats: Stats,
    /// Pontos de atributo do personagem (sem equipamento).
    pub attributes: Attributes,
    /// Soma dos itens equipados, usada por requisitos de movimento.
    pub equipment: Equipment,
    /// Itens instalados por slot. A agregação em `equipment` é derivada daqui.
    pub equipped: [Option<EquippedItem>; EquipSlot::COUNT],
    pub movement: Movement,
    pub combat: Combat,
}

impl Entity {
    pub fn is_player(&self) -> bool {
        self.kind.is_player()
    }

    pub fn is_enemy(&self) -> bool {
        self.kind.is_enemy()
    }

    pub fn is_projectile(&self) -> bool {
        self.kind.is_projectile()
    }

    pub fn is_item(&self) -> bool {
        self.kind.is_item()
    }

    pub fn is_actor(&self) -> bool {
        self.is_player() || self.is_enemy()
    }

    pub fn is_playable(&self) -> bool {
        self.controller.is_playable()
    }

    pub fn ai(&self) -> Option<&Ai> {
        self.controller.ai()
    }

    pub fn ai_mut(&mut self) -> Option<&mut Ai> {
        self.controller.ai_mut()
    }

    pub fn player(&self) -> Option<&PlayerData> {
        self.kind.player()
    }

    pub fn player_mut(&mut self) -> Option<&mut PlayerData> {
        self.kind.player_mut()
    }

    pub fn enemy(&self) -> Option<&EnemyData> {
        self.kind.enemy()
    }

    pub fn enemy_mut(&mut self) -> Option<&mut EnemyData> {
        self.kind.enemy_mut()
    }

    pub fn projectile(&self) -> Option<&ProjectileData> {
        self.kind.projectile()
    }

    pub fn projectile_mut(&mut self) -> Option<&mut ProjectileData> {
        self.kind.projectile_mut()
    }

    pub fn item(&self) -> Option<&ItemData> {
        self.kind.item()
    }

    pub fn item_mut(&mut self) -> Option<&mut ItemData> {
        self.kind.item_mut()
    }

    pub fn entity_type(&self) -> EntityType {
        self.kind.entity_type()
    }

    /// Atributos só do personagem. É o que os requisitos de itens enxergam.
    pub fn character_attribute_set(&self) -> super::attributes::AttributeSet {
        self.attributes.to_set()
    }

    /// Atributos efetivos: personagem + equipamento. É o que os requisitos de
    /// movimento enxergam.
    pub fn attribute_set(&self) -> super::attributes::AttributeSet {
        self.character_attribute_set()
            .merged(self.equipment.modifiers)
    }

    /// Equipa um item se o tipo de personagem e os atributos do personagem
    /// cumprirem as restrições declaradas pelo item. Itens de consumo não são
    /// equipáveis.
    pub fn try_equip(&mut self, item: &ItemData) -> Result<(), EquipError> {
        let slot = item.slot().ok_or(EquipError::NotEquippable)?;
        item.requirements
            .allows(self.entity_type(), self.character_attribute_set())?;
        self.equipped[slot.index()] = Some(EquippedItem {
            slot,
            kind: item.kind,
            equipment: item.equipment,
        });
        self.refresh_equipment();
        Ok(())
    }

    pub fn unequip(&mut self, slot: EquipSlot) -> Option<EquippedItem> {
        let removed = self.equipped[slot.index()].take();
        self.refresh_equipment();
        removed
    }

    /// Recalcula a agregação a partir dos slots. Chamado após qualquer mudança
    /// de equipamento.
    pub fn refresh_equipment(&mut self) {
        self.equipment = Equipment::from_equipped(&self.equipped);
    }

    pub fn ability_context(&self) -> AbilityContext {
        AbilityContext::new(
            self.entity_type(),
            self.kind.level(),
            self.attribute_set(),
            self.equipment.tags,
        )
    }

    pub fn has_move(&self, id: MoveId) -> bool {
        match id.category() {
            MoveCategory::Movement => self.movement.has(id),
            MoveCategory::Attack => self.combat.has(id),
        }
    }

    /// Atribui um movimento em runtime, revalidando os requisitos.
    pub fn try_assign_move(&mut self, id: MoveId) -> Result<(), AssignError> {
        let context = self.ability_context();
        match id.category() {
            MoveCategory::Movement => self.movement.try_assign(id, context),
            MoveCategory::Attack => self.combat.try_assign(id, context),
        }
    }

    pub fn max_health(&self) -> f32 {
        self.stats.max_health
    }

    pub fn max_aura(&self) -> f32 {
        self.stats.max_aura
    }

    pub fn aura(&self) -> f32 {
        self.player().map(|player| player.aura).unwrap_or(0.0)
    }

    pub fn rect(&self) -> Rect {
        self.body.rect()
    }

    pub fn center(&self) -> Vec2 {
        self.body.center()
    }
}
