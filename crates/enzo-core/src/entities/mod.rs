//! Componentized entity model.
//!
//! Playables, enemies, projectiles, and items are the same [`Entity`], created
//! via [`EntityStore::spawn`] with an [`EntitySpec`].
//!
//! Organization:
//! - [`base`]: core components, entity/storage, equipment, projectile data.
//! - [`actors`]: concrete actor kinds/species (Enzo/player, enemy/slime).
//! - [`item`], [`spec`], [`mogging`]: item system and entity builders/specs.

pub mod actors;
pub mod base;
pub mod item;
pub mod mogging;
pub mod spec;

pub use actors::enemy::{EnemyData, EnemySpecies};
pub use actors::enzo::ENZO_SIZE;
pub use actors::player::{PlayableCharacter, PlayerData, PlayerId, VisualProfile};
pub use actors::slime::{SLIME_CONTACT_DAMAGE, SLIME_SIZE, SLIME_XP_REWARD};
pub use base::attributes::{AttributeId, AttributeSet, Attributes, Stats};
pub use base::components::{Body, Health, TeamId, TeamRelations, MAX_TEAMS};
pub use base::controller::{Controller, PlayerController, PlayerInput};
pub use base::entity::{Entity, EntityId, EntityKind};
pub use base::equipment::{EquipSlot, Equipment, EquipmentTags, EquippedItem};
pub use base::projectile::ProjectileData;
pub use base::store::EntityStore;
pub use item::{
    EquipError, ItemCategory, ItemData, ItemKind, ItemRequirements, ItemUsage, ITEM_SIZE,
};
pub use mogging::{MOGGING_LIFETIME, MOGGING_SIZE, MOGGING_SPEED};
pub use spec::EntitySpec;

// Re-exports used by consumers.
pub use crate::mechanics::Locomotion;
pub use crate::skills::{EntityType, SkillContext, Skills};
