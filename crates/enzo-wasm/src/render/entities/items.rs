//! Render de itens no mundo.

use enzo_core::snapshot::EntitySnapshot;
use macroquad::prelude::*;

use crate::sprites::{self, Sprites};

pub fn frame(kind: enzo_core::entities::ItemKind) -> usize {
    use sprites::item as f;
    match kind {
        enzo_core::entities::ItemKind::Sword => f::SWORD,
        enzo_core::entities::ItemKind::Bow => f::BOW,
        enzo_core::entities::ItemKind::Armor => f::ARMOR,
        enzo_core::entities::ItemKind::Amulet => f::AMULET,
        enzo_core::entities::ItemKind::Potion => f::POTION,
        enzo_core::entities::ItemKind::NutellaGun => f::BOW, // placeholder visual
        enzo_core::entities::ItemKind::Sword67 => f::SWORD,
        enzo_core::entities::ItemKind::Cap67 => f::AMULET,
        enzo_core::entities::ItemKind::Shirt67 => f::ARMOR,
        enzo_core::entities::ItemKind::Sneakers67 => f::ARMOR,
    }
}

pub fn draw(sprites: &Sprites, entity: &EntitySnapshot, pos: Vec2, time: f32) {
    let Some(data) = entity.item.as_ref() else {
        return;
    };
    let size = vec2(entity.size.x, entity.size.y);
    let bob = (time * 3.0 + entity.id.0 as f32).sin() * 2.0;
    let center = pos + size * 0.5 + vec2(0.0, bob);
    draw_circle(center.x, center.y, 14.0, Color::from_rgba(246, 196, 69, 30));
    sprites.draw_item(
        frame(data.kind),
        Rect::new(center.x - 8.0, center.y - 8.0, 16.0, 16.0),
        WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use enzo_core::entities::ItemKind;

    #[test]
    fn every_item_has_a_frame() {
        assert_eq!(frame(ItemKind::Sword), sprites::item::SWORD);
        assert_eq!(frame(ItemKind::NutellaGun), sprites::item::BOW);
    }
}
