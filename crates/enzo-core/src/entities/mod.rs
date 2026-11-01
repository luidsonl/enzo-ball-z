//! Modelo componentizado de entidades.
//!
//! Jogáveis, inimigos, projéteis e itens são o mesmo [`Entity`], criado sempre
//! via [`EntityStore::spawn`] com um [`EntitySpec`].
//!
//! Componentes comuns: [`Body`], [`Health`], [`Stats`], [`Attributes`],
//! [`TeamId`], [`Controller`], [`Movement`], [`Combat`], [`Equipment`].
//! Componentes opcionais: [`crate::ai::Ai`] (inimigos) e os itens equipados.
//!
//! Cada entidade concreta tem seu arquivo: `enzo.rs` (jogável), `slime.rs`
//! (inimigo), `mogar.rs` (projétil) e `item/` (itens). Os módulos genéricos
//! (`player`, `enemy`, `projectile`, `item`) guardam só componentes e contratos.
//!
//! Requisitos de movimento enxergam personagem + equipamento; requisitos de item
//! enxergam só o personagem, como descrito em [`item`].

pub mod attributes;
pub mod components;
pub mod enemy;
pub mod entity;
pub mod enzo;
pub mod equipment;
pub mod item;
pub mod mogar;
pub mod player;
pub mod projectile;
pub mod slime;
pub mod spec;
pub mod store;

pub use attributes::{AttributeId, AttributeSet, Attributes, Stats};
pub use components::{Body, Health, TeamId, TeamRelations, MAX_TEAMS};
pub use enemy::{EnemyData, EnemySpecies};
pub use entity::{Controller, Entity, EntityId, EntityKind};
pub use enzo::ENZO_SIZE;
pub use equipment::{EquipSlot, Equipment, EquipmentTags, EquippedItem};
pub use item::{
    EquipError, ItemCategory, ItemData, ItemKind, ItemRequirements, ItemUsage, ITEM_SIZE,
};
pub use mogar::{MOGAR_LIFETIME, MOGAR_SIZE, MOGAR_SPEED};
pub use player::{PlayerData, PlayerId, PlayerInput};
pub use projectile::ProjectileData;
pub use slime::{SLIME_CONTACT_DAMAGE, SLIME_SIZE, SLIME_XP_REWARD};
pub use spec::EntitySpec;
pub use store::EntityStore;

// Reexporta o tipo de movimento para quem consome `entities`.
pub use crate::attacks::Combat;
pub use crate::movements::Movement;
