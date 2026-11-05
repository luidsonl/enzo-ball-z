//! Primitivas visuais compartilhadas pelo HUD.

use macroquad::prelude::*;

pub(super) fn panel(x: f32, y: f32, w: f32, h: f32, fill: Color, border: Color) {
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 2.0, border);
    draw_rectangle_lines(
        x + 3.0,
        y + 3.0,
        w - 6.0,
        h - 6.0,
        1.0,
        rgba(255, 255, 255, 20),
    );
}

pub(super) fn center_text(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, cx - dims.width * 0.5, y, size, color);
}

pub(super) fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}
