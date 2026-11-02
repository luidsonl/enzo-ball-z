//! Conversões entre os tipos do core e os tipos do macroquad.
//!
//! O core usa [`enzo_core::math::Vec2`] e [`enzo_core::math::Rect`]; o render usa
//! os equivalentes do macroquad. Manter a conversão num só lugar evita misturar
//! os dois tipos por acidente.

use enzo_core::math::{Rect as CoreRect, Vec2 as CoreVec2};
use macroquad::prelude::{vec2, Rect, Vec2};

pub fn to_vec2(value: CoreVec2) -> Vec2 {
    vec2(value.x, value.y)
}

pub fn to_rect(value: CoreRect) -> Rect {
    Rect::new(value.x, value.y, value.w, value.h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vectors_keep_components() {
        let converted = to_vec2(CoreVec2::new(12.0, -4.0));
        assert_eq!(converted, vec2(12.0, -4.0));
    }

    #[test]
    fn rects_keep_geometry() {
        let converted = to_rect(CoreRect::new(1.0, 2.0, 3.0, 4.0));
        assert_eq!(converted.x, 1.0);
        assert_eq!(converted.y, 2.0);
        assert_eq!(converted.w, 3.0);
        assert_eq!(converted.h, 4.0);
    }
}
