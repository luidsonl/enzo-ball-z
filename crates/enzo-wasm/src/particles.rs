//! Particle abstractions and modular emitters.
//!
//! `Effects` owns particle simulation/render data; this module defines reusable
//! spawn specs and emitters so skills/mechanics can generate particles without
//! duplicating spawn logic.

use macroquad::prelude::{vec2, Color, Vec2};
use std::f32::consts::TAU;

use crate::effects::Particle;

/// Small deterministic RNG for visual variation (not simulation state).
#[derive(Clone, Copy, Debug)]
pub struct VisualRng(pub u32);

impl VisualRng {
    pub fn next_f32(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.0 >> 8) as f32) / 16_777_216.0
    }

    pub fn range(&mut self, min: f32, max: f32) -> f32 {
        min + (max - min) * self.next_f32()
    }
}

/// Describes a single particle to spawn.
#[derive(Clone, Copy, Debug)]
pub struct ParticleSpawn {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub size: f32,
    pub color: Color,
    pub gravity: f32,
}

impl ParticleSpawn {
    pub fn into_particle(self) -> Particle {
        Particle {
            pos: self.pos,
            vel: self.vel,
            life: self.life,
            max_life: self.life,
            size: self.size,
            color: self.color,
            gravity: self.gravity,
        }
    }
}

/// Modular particle emitter/spawner.
pub trait ParticleEmitter {
    fn emit(&self, rng: &mut VisualRng, origin: Vec2, out: &mut Vec<Particle>);
}

#[derive(Clone, Copy, Debug)]
pub struct BurstEmitter {
    pub count: usize,
    pub color: Color,
    pub speed_min: f32,
    pub speed_max: f32,
    pub life_min: f32,
    pub life_max: f32,
    pub size_min: f32,
    pub size_max: f32,
    pub gravity: f32,
    pub upward_bias: f32,
    pub spread: f32, // radians, TAU = full circle
    pub base_angle: f32,
}

impl ParticleEmitter for BurstEmitter {
    fn emit(&self, rng: &mut VisualRng, origin: Vec2, out: &mut Vec<Particle>) {
        for _ in 0..self.count {
            let angle = self.base_angle + (rng.next_f32() - 0.5) * self.spread;
            let speed = rng.range(self.speed_min, self.speed_max);
            let life = rng.range(self.life_min, self.life_max);
            let size = rng.range(self.size_min, self.size_max);
            out.push(
                ParticleSpawn {
                    pos: origin,
                    vel: vec2(angle.cos() * speed, angle.sin() * speed - self.upward_bias),
                    life,
                    size,
                    color: self.color,
                    gravity: self.gravity,
                }
                .into_particle(),
            );
        }
    }
}

/// Shockwave/wave emitter: spawns particles along an arc and moves them outward
/// so they propagate as a wave front instead of a point burst.
#[derive(Clone, Copy, Debug)]
pub struct WaveEmitter {
    pub count: usize,
    pub color: Color,
    pub speed: f32,
    pub life: f32,
    pub size: f32,
    pub gravity: f32,
    pub spread: f32,
    pub base_angle: f32,
    pub start_radius: f32,
}

impl ParticleEmitter for WaveEmitter {
    fn emit(&self, _rng: &mut VisualRng, origin: Vec2, out: &mut Vec<Particle>) {
        if self.count == 0 {
            return;
        }
        let n = self.count;
        for i in 0..n {
            let t = if n == 1 {
                0.5
            } else {
                i as f32 / (n - 1) as f32
            };
            let angle = self.base_angle + (t - 0.5) * self.spread;
            let dir = vec2(angle.cos(), angle.sin());
            out.push(
                ParticleSpawn {
                    pos: origin + dir * self.start_radius,
                    vel: dir * self.speed,
                    life: self.life,
                    size: self.size,
                    color: self.color,
                    gravity: self.gravity,
                }
                .into_particle(),
            );
        }
    }
}

pub mod emitters {
    use super::*;

    pub fn sparks(color: Color, count: usize) -> BurstEmitter {
        BurstEmitter {
            count,
            color,
            speed_min: 35.0,
            speed_max: 185.0,
            life_min: 0.25,
            life_max: 0.60,
            size_min: 1.5,
            size_max: 4.0,
            gravity: 420.0,
            upward_bias: 20.0,
            spread: TAU,
            base_angle: 0.0,
        }
    }

    pub fn dust() -> BurstEmitter {
        BurstEmitter {
            count: 5,
            color: Color::from_rgba(216, 208, 192, 170),
            speed_min: 10.0,
            speed_max: 45.0,
            life_min: 0.20,
            life_max: 0.40,
            size_min: 2.0,
            size_max: 4.0,
            gravity: -40.0,
            upward_bias: 10.0,
            spread: TAU,
            base_angle: 0.0,
        }
    }

    /// Shout shockwave: propagates as an expanding arc/wave in front of the character.
    pub fn shout(facing: i8) -> WaveEmitter {
        let dir = if facing >= 0 {
            0.0
        } else {
            std::f32::consts::PI
        };
        WaveEmitter {
            count: 12,
            color: Color::from_rgba(190, 170, 255, 200),
            speed: 170.0,
            life: 0.32,
            size: 3.5,
            gravity: 0.0,
            spread: 1.6,
            base_angle: dir,
            start_radius: 10.0,
        }
    }
}
