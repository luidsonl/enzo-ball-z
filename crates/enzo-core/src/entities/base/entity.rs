//! Entidade componentizada.
//!
//! Jogável, inimigo e projétil são o mesmo [`Entity`]. O que muda é o
//! [`EntityKind`] (payload específico) e o [`Controller`] (quem decide a
//! intenção). Componentes comuns (`Body`, `Health`, `Stats`, `Attributes`,
//! `TeamId`, `Locomotion`, `Skills`) existem em todas as entidades.
//!
//! Toda criação passa por [`crate::entities::EntityStore::spawn`] com um
//! [`crate::entities::EntitySpec`].

use crate::ai::Ai;
use crate::math::{Rect, Vec2};
use crate::mechanics::Locomotion;
use crate::skills::{SkillContext, SkillId, Skills};

use super::attributes::{Attributes, Stats};
use super::components::{Body, Health, TeamId};
use super::controller::Controller;
use super::equipment::{EquipSlot, Equipment, EquippedItem};
use super::projectile::ProjectileData;
use crate::entities::actors::enemy::EnemyData;
use crate::entities::actors::player::{PlayableCharacter, PlayerData, VisualProfile};
use crate::entities::item::{EquipError, ItemData};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityId(pub u32);

/// Payload específico de cada tipo de entidade.
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

    pub fn entity_type(&self) -> crate::skills::EntityType {
        match self {
            Self::Player(_) => crate::skills::EntityType::Playable,
            Self::Enemy(_) => crate::skills::EntityType::Enemy,
            Self::Projectile(_) => crate::skills::EntityType::Projectile,
            Self::Item(_) => crate::skills::EntityType::Item,
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
    pub attributes: Attributes,
    pub equipment: Equipment,
    pub equipped: [Option<EquippedItem>; EquipSlot::COUNT],
    pub locomotion: Locomotion,
    pub skills: Skills,
    /// Visual model/character identity for humanoid rendering (players and
    /// humanoid-style enemies/variants). Stored on the entity to support
    /// multiplayer replication and varied appearances.
    pub visual_character: Option<PlayableCharacter>,
    /// Physical appearance customization (skin, hair, face, outfit, etc.).
    /// Stored on the entity so Enzo/Valentina-based enemies can have varied
    /// characteristics and appearance can be replicated in multiplayer.
    pub visual_profile: VisualProfile,
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

    pub fn entity_type(&self) -> crate::skills::EntityType {
        self.kind.entity_type()
    }

    pub fn character_attribute_set(&self) -> super::attributes::AttributeSet {
        self.attributes.to_set()
    }

    pub fn attribute_set(&self) -> super::attributes::AttributeSet {
        self.character_attribute_set()
            .merged(self.equipment.modifiers)
    }

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

    pub fn refresh_equipment(&mut self) {
        self.equipment = Equipment::from_equipped(&self.equipped);
    }

    pub fn skill_context(&self) -> SkillContext {
        SkillContext::new(
            self.entity_type(),
            self.kind.level(),
            self.attribute_set(),
            self.equipment.tags,
            self.equipment.skill_levels,
        )
    }

    pub fn has_skill(&self, id: SkillId) -> bool {
        self.skills.has(id)
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
