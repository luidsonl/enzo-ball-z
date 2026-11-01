//! Dados de projéteis.

use crate::math::Vec2;

use super::entity::EntityId;

pub const MOGAR_SIZE: Vec2 = Vec2::new(24.0, 12.0);
pub const MOGAR_SPEED: f32 = 620.0;
pub const MOGAR_LIFETIME: f32 = 1.4;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectileData {
    pub owner: EntityId,
    pub damage: f32,
    /// Alvos já atingidos. O Mogar atravessa e não acerta o mesmo alvo duas vezes.
    pub hits: Vec<EntityId>,
    pub lifetime: f32,
}
