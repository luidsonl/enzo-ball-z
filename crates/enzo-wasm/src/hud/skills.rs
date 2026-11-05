//! Conteúdo da janela de habilidades.

use enzo_core::moves::MoveId;
use enzo_core::snapshot::PlayerSnapshot;
use macroquad::prelude::*;

use crate::i18n::{I18n, Text};
use crate::ui;

use super::window::WindowState;

pub(super) fn draw(state: WindowState, player: &PlayerSnapshot, i18n: I18n, ui: f32) {
    let content = state.content_rect(ui);

    if player.moves.is_empty() {
        ui::center_text(
            i18n.t(Text::NoMoves),
            content.x + content.w * 0.5,
            content.y + content.h * 0.42,
            (13.0 * ui).max(10.0),
            ui::rgba(168, 178, 214, 230),
        );
        return;
    }

    let mut y = content.y + 8.0 * ui;

    y = draw_section(
        content,
        y,
        i18n.t(Text::Movements),
        &MoveId::MOVEMENTS,
        player,
        i18n,
        ui,
        ui::AURA,
    );
    draw_section(
        content,
        y + 8.0 * ui,
        i18n.t(Text::Attacks),
        &MoveId::ATTACKS,
        player,
        i18n,
        ui,
        ui::PURPLE,
    );
}

#[allow(clippy::too_many_arguments)]
fn draw_section(
    content: Rect,
    mut y: f32,
    heading: &str,
    ids: &[MoveId],
    player: &PlayerSnapshot,
    i18n: I18n,
    ui: f32,
    color: Color,
) -> f32 {
    ui::shadow_text(
        heading,
        content.x,
        y + 14.0 * ui,
        (13.0 * ui).max(10.0),
        color,
    );
    y += 22.0 * ui;

    let row_h = 26.0 * ui;
    for id in ids {
        if !player.moves.contains(id) {
            continue;
        }
        if y + row_h > content.y + content.h {
            return y;
        }
        draw_rectangle(content.x, y, content.w, row_h, ui::rgba(24, 26, 46, 220));
        draw_circle(content.x + 12.0 * ui, y + row_h * 0.5, 3.0 * ui, color);
        ui::shadow_text(
            i18n.movement(*id),
            content.x + 24.0 * ui,
            y + 18.0 * ui,
            (14.0 * ui).max(10.0),
            WHITE,
        );
        y += row_h + 4.0 * ui;
    }

    y
}
