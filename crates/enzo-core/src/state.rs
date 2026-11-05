//! Estado de alto nível da partida.
//!
//! Fica fora de [`crate::world`] e [`crate::snapshot`] para que ambos dependam
//! de um tipo neutro, sem ciclo entre orquestração e contrato de render/rede.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GamePhase {
    Playing,
    LevelUp,
    Dead,
    Victory,
}
