//! Render de itens no mundo (lazy, modular per-item assets).

use enzo_core::snapshot::EntitySnapshot;
use macroquad::prelude::*;

use crate::sprites::Sprites;

pub fn draw(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    let Some(data) = entity.item.as_ref() else {
        return;
    };
    let size = vec2(entity.size.x, entity.size.y);
    let bob = (time * 3.0 + entity.id.0 as f32).sin() * 2.0;
    let center = pos + size * 0.5 + vec2(0.0, bob);
    draw_circle(center.x, center.y, 14.0, Color::from_rgba(246, 196, 69, 30));
    sprites.draw_item_world(
        data.kind,
        Rect::new(center.x - 8.0, center.y - 8.0, 16.0, 16.0),
        WHITE,
    );
}

#[cfg(test)]
mod tests {
    use enzo_core::entities::ItemKind;

    #[test]
    fn all_item_kinds_have_stable_ids() {
        for kind in ItemKind::ALL {
            assert!(!kind.id().is_empty());
        }
    }
}
