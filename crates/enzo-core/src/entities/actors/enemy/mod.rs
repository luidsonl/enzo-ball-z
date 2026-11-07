//! Componente genérico de inimigos.
//!
//! Todo inimigo usa os mesmos componentes de [`crate::entities::Entity`]. A
//! espécie identifica o payload e escolhe o módulo concreto que fornece números
//! e construção (`slime.rs`, e futuros `goblin.rs`, `bat.rs` etc.).
//!
//! O comportamento de IA não fica aqui: ele é o componente
//! [`crate::ai::Ai`], com um módulo por comportamento.

/// Espécies conhecidas pelo core. Cada variante deve ter um módulo concreto em
/// `entities/` e um comportamento em `ai/`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EnemySpecies {
    Slime,
}

/// Estado comum a qualquer inimigo. A intenção de patrulha fica no componente
/// `Ai`; aqui fica só o que o combate e a progressão precisam.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enemy_data_is_a_plain_component() {
        let data = EnemyData {
            species: EnemySpecies::Slime,
            xp_reward: 10.0,
            contact_damage: 5.0,
            stun: 0.0,
            xp_awarded: false,
        };
        assert_eq!(data.species, EnemySpecies::Slime);
        assert_eq!(data.xp_reward, 10.0);
        assert!(!data.xp_awarded);
    }
}
