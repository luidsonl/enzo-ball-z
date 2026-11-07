#![allow(dead_code)]
//! Conteúdo da janela de habilidades / slots.

use enzo_core::snapshot::PlayerSnapshot;
use macroquad::prelude::*;

use crate::i18n::{I18n, Text};
use crate::ui;

use super::window::WindowState;

pub(super) fn draw(state: WindowState, player: &PlayerSnapshot, i18n: I18n, ui_scale: f32) {
    let content = state.content_rect(ui_scale);
    let mut y = content.y + 6.0 * ui_scale;
    let row_h = 28.0 * ui_scale;

    for (index, slot) in player.slots.iter().enumerate() {
        if y + row_h > content.y + content.h {
            break;
        }
        draw_rectangle(content.x, y, content.w, row_h, ui::rgba(24, 26, 46, 220));
        let label = if let Some(skill) = slot.skill {
            i18n.skill(skill)
        } else {
            i18n.t(Text::SkillEmpty)
        };
        let status = if slot.skill.is_none() {
            ""
        } else if !slot.available {
            "🔒"
        } else if slot.cooldown > 0.0 {
            "⏳"
        } else if slot.aura_cost > player.aura {
            "✦"
        } else {
            "✓"
        };
        let color = if slot.skill.is_none() || !slot.available || slot.aura_cost > player.aura {
            ui::rgba(150, 150, 170, 255)
        } else if slot.cooldown > 0.0 {
            ui::rgba(120, 112, 150, 255)
        } else {
            WHITE
        };
        ui::shadow_text(
            &format!("[{}] {}", index + 1, label),
            content.x + 8.0 * ui_scale,
            y + 18.0 * ui_scale,
            (13.0 * ui_scale).max(10.0),
            color,
        );
        ui::right_text(
            status,
            content.x + content.w - 8.0 * ui_scale,
            y + 18.0 * ui_scale,
            (12.0 * ui_scale).max(9.0),
            color,
        );
        y += row_h + 4.0 * ui_scale;
    }

    if player.slots.iter().all(|s| s.skill.is_none()) {
        ui::center_text(
            i18n.t(Text::NoMoves),
            content.x + content.w * 0.5,
            content.y + content.h * 0.5,
            (13.0 * ui_scale).max(10.0),
            ui::rgba(168, 178, 214, 230),
        );
    }
}
