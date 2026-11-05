//! Estado do ponteiro (mouse) por frame.
//!
//! O clique é edge-triggered: `pressed` vale apenas no frame em que o botão
//! esquerdo desceu. O HUD e o menu consomem esse edge para não repetir uma ação
//! enquanto o botão continua pressionado.
//!
//! A leitura do dispositivo fica isolada em [`Pointer::poll`]. O resto do
//! código recebe o ponteiro como dado e pode ser testado sem janela.

use macroquad::prelude::{
    is_mouse_button_down, is_mouse_button_pressed, is_mouse_button_released, mouse_position,
    MouseButton, Rect, Vec2,
};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Pointer {
    pub pos: Vec2,
    pub down: bool,
    pub pressed: bool,
    pub released: bool,
}

impl Pointer {
    /// Lê o mouse uma vez por frame.
    pub fn poll(&mut self) {
        let (x, y) = mouse_position();
        self.pos = Vec2::new(x, y);
        self.down = is_mouse_button_down(MouseButton::Left);
        self.pressed = is_mouse_button_pressed(MouseButton::Left);
        self.released = is_mouse_button_released(MouseButton::Left);
    }

    /// Ponteiro sintético para testes e para frames sem dispositivo.
    pub fn at(pos: Vec2) -> Self {
        Self {
            pos,
            ..Self::default()
        }
    }

    pub fn hovering(self, rect: Rect) -> bool {
        point_in_rect(self.pos, rect)
    }

    pub fn clicked(self, rect: Rect) -> bool {
        self.pressed && self.hovering(rect)
    }

    pub fn pressed_inside(self, rect: Rect) -> bool {
        self.pressed && self.hovering(rect)
    }
}

pub fn point_in_rect(point: Vec2, rect: Rect) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.w
        && point.y >= rect.y
        && point.y <= rect.y + rect.h
}

/// Mantém uma janela arrastável dentro da tela, deixando o título sempre
/// acessível.
pub fn clamp_window(pos: Vec2, size: Vec2, screen: Vec2, title_height: f32) -> Vec2 {
    let min_x = -size.x + 80.0;
    let max_x = screen.x - 80.0;
    let min_y = 0.0;
    let max_y = screen.y - title_height;
    Vec2::new(
        pos.x.clamp(min_x, max_x.max(min_x)),
        pos.y.clamp(min_y, max_y.max(min_y)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hit_testing_includes_edges() {
        let rect = Rect::new(10.0, 20.0, 100.0, 40.0);
        assert!(point_in_rect(Vec2::new(10.0, 20.0), rect));
        assert!(point_in_rect(Vec2::new(110.0, 60.0), rect));
        assert!(!point_in_rect(Vec2::new(111.0, 60.0), rect));
    }

    #[test]
    fn click_requires_a_press_inside_the_rect() {
        let rect = Rect::new(0.0, 0.0, 50.0, 50.0);
        let mut pointer = Pointer::at(Vec2::new(25.0, 25.0));
        assert!(!pointer.clicked(rect));
        pointer.pressed = true;
        assert!(pointer.clicked(rect));
        pointer.pos = Vec2::new(80.0, 25.0);
        assert!(!pointer.clicked(rect));
    }

    #[test]
    fn clamp_keeps_window_reachable() {
        let clamped = clamp_window(
            Vec2::new(-500.0, 900.0),
            Vec2::new(300.0, 200.0),
            Vec2::new(1280.0, 720.0),
            32.0,
        );
        assert_eq!(clamped, Vec2::new(-220.0, 688.0));
    }
}
