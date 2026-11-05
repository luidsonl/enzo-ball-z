//! Conteúdo da janela de inventário.

use enzo_core::snapshot::PlayerSnapshot;
use macroquad::prelude::*;

use crate::i18n::{I18n, Text};
use crate::ui;

use super::window::WindowState;

pub(super) fn draw(state: WindowState, player: &PlayerSnapshot, i18n: I18n, ui: f32) {
    let content = state.content_rect(ui);

    if player.inventory.is_empty() {
        ui::center_text(
            i18n.t(Text::InventoryEmpty),
            content.x + content.w * 0.5,
            content.y + content.h * 0.42,
            (13.0 * ui).max(10.0),
            ui::rgba(168, 178, 214, 230),
        );
        return;
    }

    let row_h = 30.0 * ui;
    let mut y = content.y;
    for (index, kind) in player.inventory.iter().enumerate() {
        if y + row_h > content.y + content.h {
            break;
        }
        draw_rectangle(
            content.x,
            y,
            content.w,
            row_h,
            if index % 2 == 0 {
                ui::rgba(24, 26, 46, 230)
            } else {
                ui::rgba(20, 22, 40, 220)
            },
        );
        ui::shadow_text(
            &(index + 1).to_string(),
            content.x + 10.0 * ui,
            y + 20.0 * ui,
            (11.0 * ui).max(8.0),
            ui::rgba(130, 138, 168, 255),
        );
        ui::shadow_text(
            i18n.item(*kind),
            content.x + 34.0 * ui,
            y + 20.0 * ui,
            (14.0 * ui).max(10.0),
            ui::GREEN,
        );
        y += row_h + 2.0 * ui;
    }
}
