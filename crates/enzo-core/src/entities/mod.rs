//! Modelo componentizado de entidades.
//!
//! Jogáveis, inimigos, projéteis e itens são o mesmo [`Entity`], criado sempre
//! via [`EntityStore::spawn`] com um [`EntitySpec`].
//!
//! Componentes comuns: [`Body`], [`Health`], [`Stats`], [`Attributes`],
//! [`TeamId`], [`Controller`], [`Locomotion`], [`Skills`], [`Equipment`].

pub mod attributes;
pub mod components;
pub mod controller;
pub mod enemy;
pub mod entity;
pub mod enzo;
pub mod equipment;
pub mod item;
pub mod mogging;
pub mod player;
pub mod projectile;
pub mod slime;
pub mod spec;
pub mod store;

pub use attributes::{AttributeId, AttributeSet, Attributes, Stats};
pub use components::{Body, Health, TeamId, TeamRelations, MAX_TEAMS};
pub use controller::{Controller, PlayerController, PlayerInput};
pub use enemy::{EnemyData, EnemySpecies};
pub use entity::{Entity, EntityId, EntityKind};
pub use enzo::ENZO_SIZE;
pub use equipment::{EquipSlot, Equipment, EquipmentTags, EquippedItem};
pub use item::{
    EquipError, ItemCategory, ItemData, ItemKind, ItemRequirements, ItemUsage, ITEM_SIZE,
};
pub use mogging::{MOGGING_LIFETIME, MOGGING_SIZE, MOGGING_SPEED};
pub use player::{PlayerData, PlayerId};
pub use projectile::ProjectileData;
pub use slime::{SLIME_CONTACT_DAMAGE, SLIME_SIZE, SLIME_XP_REWARD};
pub use spec::EntitySpec;
pub use store::EntityStore;

// Reexporta tipos usados por consumidores.
pub use crate::mechanics::Locomotion;
pub use crate::skills::{EntityType, SkillContext, Skills};
