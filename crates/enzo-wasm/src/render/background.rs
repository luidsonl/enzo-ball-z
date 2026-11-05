//! Fundo em screen space: gradiente, sol, nuvens e montanhas com parallax.

use macroquad::prelude::*;

use crate::camera::GameCamera;

const CLOUDS: &[(f32, f32, f32, f32)] = &[
    (0.05, 0.15, 0.10, 1.1),
    (0.28, 0.24, 0.16, 0.8),
    (0.52, 0.12, 0.08, 1.3),
    (0.74, 0.27, 0.20, 0.9),
    (0.92, 0.18, 0.13, 1.0),
];

/// Fundo em screen space: gradiente, sol, nuvens e montanhas com parallax.
pub struct Background {
    gradient: Texture2D,
}

impl Background {
    pub fn new() -> Self {
        // O gradiente é constante: gera uma tira 1x64 uma vez e estica na tela.
        // Desenhar 48 retângulos por frame custava 48 fill-rates de tela cheia.
        const ROWS: u16 = 64;
        let top = rgba(34, 46, 86, 255);
        let bottom = rgba(132, 176, 198, 255);
        let mut bytes = Vec::with_capacity(ROWS as usize * 4);
        for index in 0..ROWS {
            let t = index as f32 / (ROWS - 1) as f32;
            let color = mix(top, bottom, t);
            bytes.extend_from_slice(&[
                (color.r * 255.0) as u8,
                (color.g * 255.0) as u8,
                (color.b * 255.0) as u8,
                (color.a * 255.0) as u8,
            ]);
        }
        let gradient = Texture2D::from_rgba8(1, ROWS, &bytes);
        gradient.set_filter(FilterMode::Linear);
        Self { gradient }
    }
}

impl Default for Background {
    fn default() -> Self {
        Self::new()
    }
}

pub fn draw_background(background: &Background, camera: &GameCamera, time: f32) {
    let width = screen_width();
    let height = screen_height();

    draw_texture_ex(
        &background.gradient,
        0.0,
        0.0,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(width, height)),
            ..Default::default()
        },
    );

    // Sol com halo.
    let sun = vec2(width * 0.78, height * 0.17);
    draw_circle(sun.x, sun.y, height * 0.085, rgba(255, 236, 170, 26));
    draw_circle(sun.x, sun.y, height * 0.055, rgba(255, 240, 190, 52));
    draw_circle(sun.x, sun.y, height * 0.032, rgba(255, 248, 214, 150));

    // Nuvens com parallax horizontal.
    let span = width + 320.0;
    for (index, cloud) in CLOUDS.iter().enumerate() {
        let drift = time * (4.0 + index as f32 * 1.5) * cloud.3;
        let x = (cloud.0 * span - camera.center.x * cloud.2 + drift).rem_euclid(span) - 160.0;
        let y = cloud.1 * height;
        let s = cloud.3;
        let shade = if index % 2 == 0 {
            rgba(232, 236, 255, 210)
        } else {
            rgba(214, 220, 248, 190)
        };
        draw_ellipse(x, y, 44.0 * s, 15.0 * s, 0.0, shade);
        draw_ellipse(x - 26.0 * s, y + 5.0 * s, 24.0 * s, 10.0 * s, 0.0, shade);
        draw_ellipse(x + 28.0 * s, y + 6.0 * s, 26.0 * s, 11.0 * s, 0.0, shade);
        draw_ellipse(x + 4.0 * s, y - 8.0 * s, 24.0 * s, 12.0 * s, 0.0, shade);
    }

    // Montanhas distantes.
    let base = height * 0.88;
    let far = camera.center.x * 0.18;
    for index in 0..6 {
        let x = (index as f32 * width * 0.28 - far).rem_euclid(width + 500.0) - 250.0;
        draw_triangle(
            vec2(x, base),
            vec2(x + 360.0, base),
            vec2(x + 180.0, base - height * 0.30),
            rgba(74, 104, 126, 255),
        );
        draw_triangle(
            vec2(x + 120.0, base),
            vec2(x + 300.0, base),
            vec2(x + 210.0, base - height * 0.18),
            rgba(92, 126, 144, 255),
        );
    }

    // Faixa de neblina na base.
    draw_rectangle(
        0.0,
        height * 0.84,
        width,
        height * 0.16,
        rgba(180, 206, 214, 36),
    );
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

fn mix(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    Color::new(
        a.r + (b.r - a.r) * t,
        a.g + (b.g - a.g) * t,
        a.b + (b.b - a.b) * t,
        a.a + (b.a - a.a) * t,
    )
}
