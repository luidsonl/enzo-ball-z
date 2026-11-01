//! Modelo componentizado de entidades.
//!
//! Jogáveis, inimigos, projéteis e itens são o mesmo [`Entity`], criado sempre
//! via [`EntityStore::spawn`] com um [`EntitySpec`].
//!
//! Componentes comuns: [`Body`], [`Health`], [`Stats`], [`Attributes`],
//! [`TeamId`], [`Controller`], [`Movement`], [`Combat`], [`Equipment`].
//! Componentes opcionais: [`crate::ai::Ai`] (inimigos) e os itens equipados.
//!
//! Requisitos de movimento enxergam personagem + equipamento; requisitos de item
//! enxergam só o personagem, como descrito em [`item`].

pub mod attributes;
pub mod components;
pub mod enemy;
pub mod entity;
pub mod equipment;
pub mod item;
pub mod player;
pub mod projectile;
pub mod spec;
pub mod store;

pub use attributes::{AttributeId, AttributeSet, Attributes, Stats};
pub use components::{Body, Health, TeamId, TeamRelations, MAX_TEAMS};
pub use enemy::{EnemyData, EnemySpecies, ENEMY_SIZE};
pub use entity::{Controller, Entity, EntityId, EntityKind};
pub use equipment::{EquipSlot, Equipment, EquipmentTags, EquippedItem};
pub use item::{
    EquipError, ItemCategory, ItemData, ItemKind, ItemRequirements, ItemUsage, ITEM_SIZE,
};
pub use player::{AuraFarmState, PlayerData, PlayerId, PlayerInput, PLAYER_SIZE};
pub use projectile::{ProjectileData, MOGAR_LIFETIME, MOGAR_SIZE, MOGAR_SPEED};
pub use spec::EntitySpec;
pub use store::EntityStore;

// Reexporta o tipo de movimento para quem consome `entities`.
pub use crate::attacks::Combat;
pub use crate::movements::Movement;
