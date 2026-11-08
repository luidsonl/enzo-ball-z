//! Shared character composition helpers (equipped layers, customization cues).

use enzo_core::entities::{EquipSlot, ItemKind};
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::sprites::Sprites;

const DRAW_ORDER: [EquipSlot; 5] = [
    EquipSlot::Feet,
    EquipSlot::Armor,
    EquipSlot::Accessory,
    EquipSlot::Head,
    EquipSlot::Weapon,
];

pub fn draw_equipped_layers(
    sprites: &Sprites,
    player: &PlayerSnapshot,
    dest: Rect,
    flip_x: bool,
    tint: Color,
) {
    draw_equipped_layers_rig(sprites, player, None, dest, flip_x, tint);
}

/// Draws equipped item overlays. When `source` (rig frame rect) is provided, uses
/// per-animation-frame humanoid overlay sheets so items follow the current pose/state.
pub fn draw_equipped_layers_rig(
    sprites: &Sprites,
    player: &PlayerSnapshot,
    source: Option<Rect>,
    dest: Rect,
    flip_x: bool,
    tint: Color,
) {
    for slot in DRAW_ORDER {
        let Some(item) = player.equipped[slot.index()].as_ref() else {
            continue;
        };
        sprites.draw_item_equipped_rig(item.kind, source, dest, flip_x, tint);
    }
}

/// Subtle visual cue for base customization profile (placeholder for full layered rigs).
pub fn profile_tint(player: &PlayerSnapshot, base: Color) -> Color {
    let p = player.visual_profile;
    // Keep conservative: only slight hue/brightness variation so legacy sprites remain readable.
    let skin = (p.skin_tone % 4) as f32;
    let hair = (p.hair_color % 4) as f32;
    let r = (base.r + skin * 0.015 + hair * 0.005).clamp(0.0, 1.0);
    let g = (base.g + hair * 0.01).clamp(0.0, 1.0);
    let b = (base.b + (3.0 - skin) * 0.01).clamp(0.0, 1.0);
    Color::new(r, g, b, base.a)
}

pub fn is_equipped(player: &PlayerSnapshot, kind: ItemKind) -> bool {
    player
        .equipped
        .iter()
        .flatten()
        .any(|item| item.kind == kind)
}

pub fn animation(entity: &EntitySnapshot) -> enzo_core::entities::AnimationState {
    entity.animation
}
