//! Janelas arrastáveis do HUD.
//!
//! Cada janela tem estado próprio (aberta, posição e tamanho). O conteúdo vive
//! em módulos separados (`status_window`, `inventory`, `equipment`, `skills`);
//! este módulo cuida apenas da moldura, do título e do botão de fechar.

use macroquad::prelude::*;

use crate::i18n::{I18n, Text};
use crate::pointer::Pointer;
use crate::ui::{self, rgba};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WindowId {
    Status,
    Inventory,
    Equipment,
    Skills,
}

impl WindowId {
    pub const ALL: [WindowId; 4] = [
        WindowId::Status,
        WindowId::Inventory,
        WindowId::Equipment,
        WindowId::Skills,
    ];

    pub fn index(self) -> usize {
        match self {
            Self::Status => 0,
            Self::Inventory => 1,
            Self::Equipment => 2,
            Self::Skills => 3,
        }
    }

    pub fn title(self, i18n: I18n) -> &'static str {
        match self {
            Self::Status => i18n.t(Text::StatusTitle),
            Self::Inventory => i18n.t(Text::InventoryTitle),
            Self::Equipment => i18n.t(Text::EquipmentTitle),
            Self::Skills => i18n.t(Text::SkillsTitle),
        }
    }

    pub fn accent(self) -> Color {
        match self {
            Self::Status => ui::GOLD,
            Self::Inventory => ui::GREEN,
            Self::Equipment => ui::STEEL,
            Self::Skills => ui::PURPLE,
        }
    }

    /// Inicial usada nos dots e no selo da janela. Sai do título localizado,
    /// então acompanha o idioma ativo.
    pub fn short(self, i18n: I18n) -> &'static str {
        let title = self.title(i18n);
        &title[..1]
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WindowState {
    pub open: bool,
    pub pos: Vec2,
    pub size: Vec2,
}

impl WindowState {
    pub fn new(pos: Vec2, size: Vec2) -> Self {
        Self {
            open: false,
            pos,
            size,
        }
    }

    pub fn rect(self) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, self.size.y)
    }

    pub fn title_height(ui: f32) -> f32 {
        34.0 * ui
    }

    pub fn title_rect(self, ui: f32) -> Rect {
        Rect::new(self.pos.x, self.pos.y, self.size.x, Self::title_height(ui))
    }

    pub fn close_rect(self, ui: f32) -> Rect {
        let size = 22.0 * ui;
        Rect::new(
            self.pos.x + self.size.x - size - 7.0 * ui,
            self.pos.y + 6.0 * ui,
            size,
            size,
        )
    }

    pub fn content_rect(self, ui: f32) -> Rect {
        let title = Self::title_height(ui);
        Rect::new(
            self.pos.x + 14.0 * ui,
            self.pos.y + title + 12.0 * ui,
            self.size.x - 28.0 * ui,
            self.size.y - title - 24.0 * ui,
        )
    }
}

/// Moldura da janela. O foco muda a cor da borda, mas não o conteúdo.
pub fn draw_chrome(
    state: WindowState,
    id: WindowId,
    i18n: I18n,
    ui: f32,
    pointer: Pointer,
    focused: bool,
) {
    let accent = id.accent();
    let border = if focused {
        accent
    } else {
        rgba(
            (accent.r * 255.0) as u8,
            (accent.g * 255.0) as u8,
            (accent.b * 255.0) as u8,
            80,
        )
    };

    ui::panel(
        state.pos.x,
        state.pos.y,
        state.size.x,
        state.size.y,
        rgba(12, 14, 30, 248),
        border,
    );

    let title_rect = state.title_rect(ui);
    draw_rectangle(
        title_rect.x + 3.0,
        title_rect.y + 3.0,
        title_rect.w - 6.0,
        title_rect.h - 3.0,
        rgba(24, 26, 48, 250),
    );
    draw_rectangle(
        title_rect.x + 3.0,
        title_rect.y + title_rect.h - 2.0,
        title_rect.w - 6.0,
        2.0,
        border,
    );

    ui::shadow_text(
        id.title(i18n),
        title_rect.x + 13.0 * ui,
        title_rect.y + 23.0 * ui,
        (14.0 * ui).max(10.0),
        accent,
    );

    // Bolinha com a inicial, para identificar a janela quando o título é longo.
    draw_circle(
        title_rect.x + title_rect.w - 72.0 * ui,
        title_rect.y + title_rect.h * 0.5,
        7.0 * ui,
        accent,
    );
    ui::center_text(
        id.short(i18n),
        title_rect.x + title_rect.w - 72.0 * ui,
        title_rect.y + title_rect.h * 0.5 + 4.0 * ui,
        (10.0 * ui).max(8.0),
        rgba(16, 16, 28, 255),
    );

    let close = state.close_rect(ui);
    let hovered = pointer.hovering(close);
    draw_rectangle(
        close.x,
        close.y,
        close.w,
        close.h,
        if hovered {
            rgba(232, 72, 63, 230)
        } else {
            rgba(44, 46, 70, 220)
        },
    );
    let size = (15.0 * ui).max(11.0);
    ui::center_text(
        "×",
        close.x + close.w * 0.5,
        close.y + close.h * 0.5 + size * 0.36,
        size,
        WHITE,
    );

    // Dica discreta de arraste no rodapé.
    ui::center_text(
        i18n.t(Text::DragHint),
        state.pos.x + state.size.x * 0.5,
        state.pos.y + state.size.y - 7.0 * ui,
        (10.0 * ui).max(8.0),
        rgba(130, 138, 168, 170),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_rects_are_consistent() {
        let state = WindowState::new(Vec2::new(100.0, 50.0), Vec2::new(300.0, 200.0));
        assert_eq!(state.rect(), Rect::new(100.0, 50.0, 300.0, 200.0));
        assert!(state.title_rect(1.0).h < state.rect().h);
        let content = state.content_rect(1.0);
        assert!(content.y > state.title_rect(1.0).y);
        assert!(content.w < state.rect().w);
        let close = state.close_rect(1.0);
        assert!(close.x > state.pos.x + state.size.x * 0.5);
        assert!(crate::pointer::point_in_rect(
            Vec2::new(close.x + 1.0, close.y + 1.0),
            state.title_rect(1.0),
        ));
    }

    #[test]
    fn window_ids_have_distinct_indices_and_titles() {
        let i18n = I18n::default();
        for (index, id) in WindowId::ALL.iter().enumerate() {
            assert_eq!(id.index(), index);
            assert!(!id.short(i18n).is_empty());
        }
        assert_eq!(i18n.t(Text::StatusTitle), WindowId::Status.title(i18n));
    }
}
