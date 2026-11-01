//! Inimigos: dados específicos por espécie.
//!
//! Todo inimigo usa os mesmos componentes de [`crate::entities::Entity`]. A
//! espécie só escolhe os números base e qual comportamento de IA será anexado
//! pelo [`crate::entities::EntitySpec`].

use crate::math::Vec2;

pub const ENEMY_SIZE: Vec2 = Vec2::new(26.0, 26.0);

/// Espécies conhecidas pelo core. Novas espécies entram como variante e ganham
/// um construtor em [`EnemyData`] + um comportamento em `crate::ai`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EnemySpecies {
    Slime,
}

impl EnemySpecies {
    pub fn xp_reward(self) -> f32 {
        match self {
            Self::Slime => 25.0,
        }
    }

    pub fn contact_damage(self) -> f32 {
        match self {
            Self::Slime => 10.0,
        }
    }
}

/// Estado de um inimigo. A intenção de patrulha fica no componente `Ai`; aqui
/// fica só o que é comum a qualquer comportamento (espécie, recompensa, dano de
/// contato e atordoamento).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemyData {
    pub species: EnemySpecies,
    pub xp_reward: f32,
    pub contact_damage: f32,
    pub stun: f32,
    /// Marca que o XP da morte já foi concedido. Torna `process_kills`
    /// idempotente mesmo antes do despawn.
    pub xp_awarded: bool,
}

impl EnemyData {
    pub fn slime() -> Self {
        Self {
            species: EnemySpecies::Slime,
            xp_reward: EnemySpecies::Slime.xp_reward(),
            contact_damage: EnemySpecies::Slime.contact_damage(),
            stun: 0.0,
            xp_awarded: false,
        }
    }
}

impl Default for EnemyData {
    fn default() -> Self {
        Self::slime()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slime_has_simple_melee_stats() {
        let slime = EnemyData::slime();
        assert_eq!(slime.species, EnemySpecies::Slime);
        assert_eq!(slime.xp_reward, 25.0);
        assert_eq!(slime.contact_damage, 10.0);
        assert_eq!(slime.stun, 0.0);
        assert!(!slime.xp_awarded);
    }
}
