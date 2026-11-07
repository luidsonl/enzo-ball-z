//! Componente genérico de projéteis.

use crate::skills::SkillId;

use super::entity::EntityId;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectileData {
    pub owner: EntityId,
    pub skill: SkillId,
    pub damage: f32,
    /// Alvos já atingidos por este projétil.
    pub hits: Vec<EntityId>,
    pub lifetime: f32,
}
