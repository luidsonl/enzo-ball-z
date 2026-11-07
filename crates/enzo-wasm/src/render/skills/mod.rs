//! Render/animação de skills (projéteis e overlays de ataque).
//!
//! `render/world.rs` orquestra; cada skill decide como desenhar seus efeitos.

pub mod melee;
pub mod mogging;
pub mod nutella_shot;

use enzo_core::skills::SkillId;
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::sprites::Sprites;

/// Desenha um projétil de acordo com a skill que o gerou.
pub fn draw_projectile(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    let Some(proj) = entity.projectile.as_ref() else {
        return;
    };
    match proj.skill {
        SkillId::Mogging => mogging::draw_projectile(sprites, entity, pos, time),
        SkillId::NutellaShot => nutella_shot::draw_projectile(sprites, entity, pos, time),
        _ => mogging::draw_projectile(sprites, entity, pos, time), // fallback
    }
}

/// Overlays de ataque/skill desenhados sobre o jogador.
pub fn draw_player_overlays(
    sprites: &Sprites,
    entity: &EntitySnapshot,
    player: &PlayerSnapshot,
    center: Vec2,
) {
    melee::draw_overlay(sprites, entity, player, center);
    // Shout/outros overlays podem entrar aqui.
}
