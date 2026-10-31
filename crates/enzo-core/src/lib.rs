//! Simulação pura do EnzoBallZ.
//!
//! Este crate não conhece render, teclado, arquivos, rede ou relógio. Ele recebe
//! [`input::TickInput`] e avança o mundo em passos fixos de `1/60 s`.

pub mod aura;
pub mod combat;
pub mod entity;
pub mod events;
pub mod input;
pub mod levels;
pub mod map;
pub mod math;
pub mod physics;
pub mod rng;

pub const TICK_HZ: u32 = 60;
pub const DT: f32 = 1.0 / 60.0;
pub const MAX_PLAYERS: usize = 4;
