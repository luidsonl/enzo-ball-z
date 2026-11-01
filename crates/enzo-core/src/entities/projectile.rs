//! Componente genérico de projéteis.
//!
//! Projéteis concretos (como o Mogar) ficam em módulos próprios e definem seus
//! números e construção. Aqui fica só o payload comum ao combate.

use super::entity::EntityId;

#[derive(Clone, Debug, PartialEq)]
pub struct ProjectileData {
    pub owner: EntityId,
    pub damage: f32,
    /// Alvos já atingidos. O Mogar atravessa e não acerta o mesmo alvo duas vezes.
    pub hits: Vec<EntityId>,
    pub lifetime: f32,
}
