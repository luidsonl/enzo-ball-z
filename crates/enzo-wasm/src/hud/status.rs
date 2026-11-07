#![allow(dead_code)]
//! Barra fixa do jogador e toggles dos submenus.
//!
//! A barra nunca se move. Os quatro dots à direita abrem/fecham as janelas
//! arrastáveis; o dot de status pisca enquanto houver level-up pendente.

use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::effects::Effects;
use crate::i18n::{I18n, Text};
use crate::pointer::Pointer;
use crate::ui;

use super::window::{WindowId, WindowState};

pub(super) struct FixedHud {
    pub toggle: Option<WindowId>,
    pub menu: bool,
}

#[allow(clippy::too_many_arguments)]
pub(super) fn draw_fixed(
    entity: &EntitySnapshot,
    player: &PlayerSnapshot,
    effects: &Effects,
    i18n: I18n,
    pointer: Pointer,
    time: f32,
    ui: f32,
    windows: &[WindowState; WindowId::ALL.len()],
) -> FixedHud {
    let x = 16.0 * ui;
    let y = 16.0 * ui;
    let w = 404.0 * ui;
    let h = 132.0 * ui;
    let pad = 12.0 * ui;

    ui::panel(
        x,
        y,
        w,
        h,
        ui::rgba(12, 12, 26, 218),
        ui::rgba(94, 242, 255, 70),
    );

    ui::shadow_text("ENZO", x + pad, y + 26.0 * ui, (22.0 * ui).max(15.0), WHITE);
    let level = format!("{} {}", i18n.t(Text::LevelShort), player.level);
    ui::shadow_text(
        &level,
        x + pad + 80.0 * ui,
        y + 25.0 * ui,
        (17.0 * ui).max(12.0),
        ui::GOLD,
    );

    if player.points > 0 {
        let pulse = 0.5 + 0.5 * (time * 5.0).sin();
        let points = format!("{} {}", player.points, i18n.t(Text::Points));
        ui::right_text(
            &points,
            x + w - 40.0 * ui,
            y + 25.0 * ui,
            (12.0 * ui).max(9.0),
            ui::rgba(255, 227, 138, (170.0 + 85.0 * pulse) as u8),
        );
    }

    let bar_x = x + pad;
    let bar_w = w - pad * 2.0 - 32.0 * ui;
    let hp_ratio = (entity.health / entity.max_health.max(1.0)).clamp(0.0, 1.0);
    let aura_ratio = (player.aura / player.max_aura.max(1.0)).clamp(0.0, 1.0);
    let xp_ratio = (player.xp / player.xp_to_next.max(1.0)).clamp(0.0, 1.0);

    draw_bar(
        bar_x,
        y + 38.0 * ui,
        bar_w,
        14.0 * ui,
        hp_ratio,
        ui::rgba(232, 72, 63, 255),
        ui::rgba(255, 128, 116, 255),
    );
    bar_label(
        bar_x,
        y + 38.0 * ui,
        bar_w,
        14.0 * ui,
        &format!(
            "{}  {:.0}/{:.0}",
            i18n.t(Text::Health),
            entity.health,
            entity.max_health
        ),
        ui,
    );

    let aura_flash = effects.aura_flash > 0.0;
    draw_bar(
        bar_x,
        y + 58.0 * ui,
        bar_w,
        14.0 * ui,
        aura_ratio,
        if aura_flash {
            ui::rgba(220, 255, 255, 255)
        } else {
            ui::rgba(58, 196, 220, 255)
        },
        ui::rgba(94, 242, 255, 255),
    );
    bar_label(
        bar_x,
        y + 58.0 * ui,
        bar_w,
        14.0 * ui,
        &format!(
            "{}  {:.0}/{:.0}",
            i18n.t(Text::Aura),
            player.aura,
            player.max_aura
        ),
        ui,
    );

    draw_bar(
        bar_x,
        y + 78.0 * ui,
        bar_w,
        10.0 * ui,
        xp_ratio,
        ui::rgba(196, 148, 34, 255),
        ui::rgba(246, 196, 69, 255),
    );
    bar_label(
        bar_x,
        y + 78.0 * ui,
        bar_w,
        10.0 * ui,
        &format!(
            "{}  {:.0}/{:.0}",
            i18n.t(Text::Xp),
            player.xp,
            player.xp_to_next
        ),
        ui,
    );

    // Barra de slots (simplificada)
    let slot_y = y + 96.0 * ui;
    let slot_w = (bar_w - 9.0 * ui) / 4.0;
    for (i, slot) in player.slots.iter().enumerate() {
        let sx = bar_x + i as f32 * (slot_w + 3.0 * ui);
        let ready = slot.skill.is_some()
            && slot.available
            && slot.cooldown <= 0.0
            && slot.aura_cost <= player.aura;
        let bg = if slot.skill.is_none() {
            ui::rgba(30, 30, 45, 220)
        } else if ready {
            ui::rgba(60, 50, 90, 230)
        } else {
            ui::rgba(35, 32, 50, 220)
        };
        draw_rectangle(sx, slot_y, slot_w, 24.0 * ui, bg);
        draw_rectangle_lines(
            sx,
            slot_y,
            slot_w,
            24.0 * ui,
            1.0,
            ui::rgba(94, 242, 255, 50),
        );
        let label = slot.skill.map(|s| i18n.skill(s)).unwrap_or("-");
        let short: String = label.chars().take(6).collect();
        ui::center_text(
            &format!("{} {}", i + 1, short),
            sx + slot_w * 0.5,
            slot_y + 16.0 * ui,
            (9.0 * ui).max(7.0),
            if ready {
                WHITE
            } else {
                ui::rgba(150, 150, 170, 255)
            },
        );
    }

    // Dots de submenu. A coluna fica dentro da barra fixa.
    let dot_x = x + w - 19.0 * ui;
    let mut toggle = None;
    for (index, id) in WindowId::ALL.iter().enumerate() {
        let center = Vec2::new(dot_x, y + 27.0 * ui + index as f32 * 24.0 * ui);
        let open = windows[id.index()].open;
        let pulse = if *id == WindowId::Status && (player.pending_level_up || player.points > 0) {
            time
        } else {
            0.0
        };
        if ui::dot_button(pointer, center, 7.0 * ui, open, true, id.accent(), pulse) {
            toggle = Some(*id);
        }
        ui::center_text(
            id.short(i18n),
            center.x,
            center.y + 3.4 * ui,
            (9.0 * ui).max(7.0),
            if open {
                ui::rgba(16, 16, 28, 255)
            } else {
                WHITE
            },
        );
    }

    let menu_rect = Rect::new(screen_width() - 108.0 * ui, y, 92.0 * ui, 30.0 * ui);
    let menu = ui::button(
        pointer,
        menu_rect,
        i18n.t(Text::MenuButton),
        ui,
        true,
        ui::AURA,
    );

    FixedHud { toggle, menu }
}

fn draw_bar(x: f32, y: f32, w: f32, h: f32, ratio: f32, fill: Color, light: Color) {
    draw_rectangle(x - 1.0, y - 1.0, w + 2.0, h + 2.0, ui::rgba(4, 4, 12, 220));
    draw_rectangle(x, y, w, h, ui::rgba(38, 38, 58, 230));
    let filled = (w * ratio.clamp(0.0, 1.0)).max(0.0);
    if filled > 0.0 {
        draw_rectangle(x, y, filled, h, fill);
        draw_rectangle(x, y, filled, (h * 0.35).max(1.0), light);
    }
    draw_rectangle_lines(x, y, w, h, 1.0, ui::rgba(150, 160, 200, 90));
}

fn bar_label(x: f32, y: f32, w: f32, h: f32, text: &str, ui: f32) {
    let size = (10.0 * ui).max(8.0);
    let dims = measure_text(text, None, size as u16, 1.0);
    let tx = x + (w - dims.width) * 0.5;
    let ty = y + h * 0.5 + size * 0.36;
    draw_text(text, tx + 1.0, ty + 1.0, size, ui::rgba(4, 4, 12, 220));
    draw_text(text, tx, ty, size, WHITE);
}
