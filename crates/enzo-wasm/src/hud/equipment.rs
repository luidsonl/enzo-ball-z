#![allow(dead_code)]
//! Conteúdo da janela de equipamentos.

use enzo_core::entities::EquipSlot;
use enzo_core::snapshot::PlayerSnapshot;
use macroquad::prelude::*;

use crate::i18n::{I18n, Text};
use crate::ui;

use super::window::WindowState;

const SLOTS: [EquipSlot; EquipSlot::COUNT] =
    [EquipSlot::Weapon, EquipSlot::Armor, EquipSlot::Accessory];

pub(super) fn draw(state: WindowState, player: &PlayerSnapshot, i18n: I18n, ui: f32) {
    let content = state.content_rect(ui);
    let row_h = 46.0 * ui;
    let mut y = content.y;

    for slot in SLOTS {
        draw_rectangle(content.x, y, content.w, row_h, ui::rgba(22, 24, 44, 230));
        draw_rectangle_lines(
            content.x,
            y,
            content.w,
            row_h,
            1.0,
            ui::rgba(94, 242, 255, 40),
        );

        ui::shadow_text(
            i18n.equip_slot(slot),
            content.x + 12.0 * ui,
            y + 18.0 * ui,
            (12.0 * ui).max(9.0),
            ui::rgba(168, 178, 214, 255),
        );

        let item = player.equipped[slot.index()].as_ref();
        let (label, color) = match item {
            Some(item) => (i18n.item(item.kind), ui::STEEL),
            None => (i18n.t(Text::NoItem), ui::rgba(120, 112, 150, 255)),
        };
        ui::shadow_text(
            label,
            content.x + 12.0 * ui,
            y + 36.0 * ui,
            (15.0 * ui).max(11.0),
            color,
        );

        y += row_h + 6.0 * ui;
    }
}
