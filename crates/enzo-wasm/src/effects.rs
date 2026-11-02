//! Efeitos visuais efêmeros: números de dano, faíscas, flashes e tremor.
//!
//! Nada aqui altera a simulação. Tudo é alimentado pelos eventos do core e
//! expira em tempo de render.

use macroquad::prelude::{vec2, Color, Vec2};
use std::f32::consts::TAU;

pub struct FloatingText {
    pub text: String,
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub color: Color,
    pub size: f32,
}

pub struct Particle {
    pub pos: Vec2,
    pub vel: Vec2,
    pub life: f32,
    pub max_life: f32,
    pub size: f32,
    pub color: Color,
    pub gravity: f32,
}

pub struct Effects {
    pub texts: Vec<FloatingText>,
    pub particles: Vec<Particle>,
    pub hurt_flash: f32,
    pub aura_flash: f32,
    pub victory_flash: f32,
    rng: u32,
}

impl Effects {
    pub fn new() -> Self {
        Self {
            texts: Vec::new(),
            particles: Vec::new(),
            hurt_flash: 0.0,
            aura_flash: 0.0,
            victory_flash: 0.0,
            rng: 0x5eed_1234,
        }
    }

    pub fn clear(&mut self) {
        self.texts.clear();
        self.particles.clear();
        self.hurt_flash = 0.0;
        self.aura_flash = 0.0;
        self.victory_flash = 0.0;
    }

    pub fn update(&mut self, dt: f32) {
        self.hurt_flash = (self.hurt_flash - dt).max(0.0);
        self.aura_flash = (self.aura_flash - dt).max(0.0);
        self.victory_flash = (self.victory_flash - dt).max(0.0);

        for text in &mut self.texts {
            text.life -= dt;
            text.pos += text.vel * dt;
            text.vel.y += 18.0 * dt;
        }
        self.texts.retain(|text| text.life > 0.0);

        for particle in &mut self.particles {
            particle.life -= dt;
            particle.vel.y += particle.gravity * dt;
            particle.pos += particle.vel * dt;
        }
        self.particles.retain(|particle| particle.life > 0.0);
    }

    pub fn text(&mut self, pos: Vec2, text: impl Into<String>, color: Color, size: f32) {
        self.texts.push(FloatingText {
            text: text.into(),
            pos,
            vel: vec2(0.0, -30.0),
            life: 0.95,
            max_life: 0.95,
            color,
            size,
        });
    }

    pub fn damage(&mut self, pos: Vec2, amount: f32, color: Color) {
        self.text(pos, format!("-{amount:.0}"), color, 11.0);
    }

    pub fn xp(&mut self, pos: Vec2, amount: f32) {
        self.text(
            pos,
            format!("+{amount:.0} XP"),
            Color::from_rgba(246, 196, 69, 255),
            10.0,
        );
    }

    pub fn sparks(&mut self, pos: Vec2, color: Color, count: usize) {
        for _ in 0..count {
            let angle = self.next_f32() * TAU;
            let speed = 35.0 + self.next_f32() * 150.0;
            let life = 0.25 + self.next_f32() * 0.35;
            let size = 1.5 + self.next_f32() * 2.5;
            self.particles.push(Particle {
                pos,
                vel: vec2(angle.cos() * speed, angle.sin() * speed - 20.0),
                life,
                max_life: life,
                size,
                color,
                gravity: 420.0,
            });
        }
    }

    pub fn dust(&mut self, pos: Vec2) {
        for _ in 0..5 {
            let angle = self.next_f32() * TAU;
            let speed = 10.0 + self.next_f32() * 35.0;
            let life = 0.2 + self.next_f32() * 0.2;
            let size = 2.0 + self.next_f32() * 2.0;
            self.particles.push(Particle {
                pos,
                vel: vec2(angle.cos() * speed, angle.sin() * speed - 10.0),
                life,
                max_life: life,
                size,
                color: Color::from_rgba(216, 208, 192, 170),
                gravity: -40.0,
            });
        }
    }

    pub fn hurt(&mut self, pos: Vec2) {
        self.hurt_flash = 0.28;
        self.sparks(pos, Color::from_rgba(255, 107, 94, 255), 7);
    }

    pub fn no_aura(&mut self, pos: Vec2) {
        self.aura_flash = 0.45;
        self.text(pos, "Sem aura!", Color::from_rgba(94, 242, 255, 255), 10.0);
    }

    fn next_f32(&mut self) -> f32 {
        self.rng = self.rng.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        ((self.rng >> 8) as f32) / 16_777_216.0
    }
}

impl Default for Effects {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn floating_text_expires() {
        let mut effects = Effects::new();
        effects.text(Vec2::ZERO, "teste", Color::new(1.0, 1.0, 1.0, 1.0), 10.0);
        assert_eq!(effects.texts.len(), 1);
        for _ in 0..120 {
            effects.update(1.0 / 60.0);
        }
        assert!(effects.texts.is_empty());
    }

    #[test]
    fn particles_expire_and_timers_decay() {
        let mut effects = Effects::new();
        effects.hurt(Vec2::ZERO);
        effects.no_aura(Vec2::ZERO);
        assert!(effects.hurt_flash > 0.0);
        assert!(effects.aura_flash > 0.0);
        assert!(!effects.particles.is_empty());

        for _ in 0..120 {
            effects.update(1.0 / 60.0);
        }
        assert_eq!(effects.hurt_flash, 0.0);
        assert_eq!(effects.aura_flash, 0.0);
        assert!(effects.particles.is_empty());
    }

    #[test]
    fn clear_removes_everything() {
        let mut effects = Effects::new();
        effects.damage(Vec2::ZERO, 12.0, Color::new(1.0, 1.0, 1.0, 1.0));
        effects.sparks(Vec2::ZERO, Color::new(1.0, 1.0, 1.0, 1.0), 4);
        effects.victory_flash = 1.0;
        effects.clear();
        assert!(effects.texts.is_empty());
        assert!(effects.particles.is_empty());
        assert_eq!(effects.victory_flash, 0.0);
    }
}
