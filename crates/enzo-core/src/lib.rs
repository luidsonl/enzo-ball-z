//! Simulação pura do EnzoBallZ.
//!
//! Este crate não conhece render, teclado, arquivos, rede ou relógio. Ele recebe
//! [`input::TickInput`] e avança o mundo em passos fixos de `1/60 s`.

pub mod ai;
pub mod attacks;
pub mod aura;
pub mod entities;
pub mod events;
pub mod input;
pub mod levels;
pub mod map;
pub mod math;
pub mod movements;
pub mod moves;
pub mod physics;
pub mod progression;
pub mod rng;
pub mod snapshot;
pub mod world;

pub const TICK_HZ: u32 = 60;
pub const DT: f32 = 1.0 / 60.0;
pub const MAX_PLAYERS: usize = 4;
