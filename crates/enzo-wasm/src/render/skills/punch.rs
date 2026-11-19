//! Visual do ataque corpo a corpo (soco).

use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::sprites::Sprites;

pub fn draw_overlay(
    _sprites: &Sprites,
    _entity: &EntitySnapshot,
    player: &PlayerSnapshot,
    _center: Vec2,
) {
    // Punch should not emit an impact animation in-world.
    let _ = player.melee_timer;
}
