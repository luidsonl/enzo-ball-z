//! Primitivas visuais compartilhadas por menu e HUD.
//!
//! Tudo aqui é screen space e imediato: desenha e devolve se o ponteiro clicou.
//! Nenhuma regra de jogo ou estado persistente vive neste módulo.

use macroquad::prelude::*;

use crate::pointer::Pointer;

pub const AURA: Color = Color::new(0.369, 0.949, 1.0, 1.0);
pub const GOLD: Color = Color::new(0.965, 0.769, 0.271, 1.0);
pub const RED: Color = Color::new(1.0, 0.42, 0.37, 1.0);
pub const PURPLE: Color = Color::new(0.722, 0.545, 1.0, 1.0);
pub const STEEL: Color = Color::new(0.82, 0.86, 0.94, 1.0);
pub const GREEN: Color = Color::new(0.561, 0.878, 0.478, 1.0);

/// Escala da UI a partir de uma altura de referência de 720 px.
/// O multiplicador de fonte permite aumentar/reduzir o tamanho do texto.
static mut FONT_SCALE: f32 = 1.25;

pub fn set_font_scale(scale: f32) {
    unsafe {
        FONT_SCALE = scale.clamp(0.75, 2.0);
    }
}

pub fn font_scale() -> f32 {
    unsafe { FONT_SCALE }
}

pub fn ui_scale() -> f32 {
    (screen_height() / 720.0).clamp(0.8, 2.0)
}

pub fn screen_size() -> Vec2 {
    Vec2::new(screen_width(), screen_height())
}

pub fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

pub fn panel(x: f32, y: f32, w: f32, h: f32, fill: Color, border: Color) {
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

pub fn center_text(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, cx - dims.width * 0.5, y, size, color);
}

pub fn shadow_text(text: &str, x: f32, y: f32, size: f32, color: Color) {
    draw_text(text, x + 1.0, y + 1.0, size, rgba(4, 4, 12, 220));
    draw_text(text, x, y, size, color);
}

pub fn right_text(text: &str, right: f32, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    shadow_text(text, right - dims.width, y, size, color);
}

/// Botão retangular com hover. Retorna `true` apenas no frame do clique.
pub fn button(
    pointer: Pointer,
    rect: Rect,
    label: &str,
    ui: f32,
    enabled: bool,
    accent: Color,
) -> bool {
    let hovered = enabled && pointer.hovering(rect);
    let fill = if !enabled {
        rgba(18, 18, 32, 205)
    } else if hovered {
        rgba(44, 44, 78, 245)
    } else {
        rgba(26, 26, 48, 232)
    };
    let text_color = if enabled {
        WHITE
    } else {
        rgba(120, 124, 150, 255)
    };
    let border = if !enabled {
        rgba(80, 84, 108, 80)
    } else if hovered {
        GOLD
    } else {
        accent
    };

    draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
    draw_rectangle_lines(
        rect.x,
        rect.y,
        rect.w,
        rect.h,
        if hovered { 2.0 } else { 1.0 },
        border,
    );
    let size = ((15.0 * ui) * font_scale()).max(11.0);
    center_text(
        label,
        rect.x + rect.w * 0.5,
        rect.y + rect.h * 0.5 + size * 0.36,
        size,
        text_color,
    );

    enabled && pointer.clicked(rect)
}

/// Botão circular usado pelos toggles do HUD. O centro e o raio são em screen
/// space.
pub fn dot_button(
    pointer: Pointer,
    center: Vec2,
    radius: f32,
    open: bool,
    enabled: bool,
    accent: Color,
    pulse: f32,
) -> bool {
    let hit = pointer.pos.distance(center) <= radius + 3.0;
    let hovered = enabled && hit;
    let glow = if pulse > 0.0 {
        0.5 + 0.5 * (pulse * 7.0).sin()
    } else {
        0.0
    };

    if open || glow > 0.0 {
        draw_circle(
            center.x,
            center.y,
            radius + 5.0 + glow * 3.0,
            rgba(
                (accent.r * 255.0) as u8,
                (accent.g * 255.0) as u8,
                (accent.b * 255.0) as u8,
                (30.0 + glow * 45.0) as u8,
            ),
        );
    }

    let fill = if open {
        accent
    } else if hovered {
        rgba(52, 54, 82, 255)
    } else if enabled {
        rgba(28, 30, 50, 245)
    } else {
        rgba(20, 20, 32, 220)
    };
    draw_circle(center.x, center.y, radius, fill);
    draw_circle_lines(
        center.x,
        center.y,
        radius,
        if hovered { 2.0 } else { 1.0 },
        if enabled {
            accent
        } else {
            rgba(80, 84, 108, 180)
        },
    );

    enabled && pointer.pressed && hit
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn colors_are_opaque() {
        assert_eq!(AURA.a, 1.0);
        assert_eq!(GOLD.a, 1.0);
        assert_eq!(rgba(1, 2, 3, 4).a, 4.0 / 255.0);
    }
}
