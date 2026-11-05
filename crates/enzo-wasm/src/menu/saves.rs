//! Tela de gerenciamento de saves.

use macroquad::prelude::*;

use super::{slot_summary, Menu, MenuEvent};
use crate::i18n::{I18n, Text};
use crate::pointer::Pointer;
use crate::save;
use crate::ui::{self, rgba, GOLD};

pub(super) fn draw(
    pointer: Pointer,
    i18n: I18n,
    ui: f32,
    _time: f32,
    menu: &Menu,
) -> Option<MenuEvent> {
    let width = (screen_width() * 0.84).clamp(420.0 * ui, 780.0 * ui);
    let height = (screen_height() * 0.82).clamp(410.0 * ui, 560.0 * ui);
    let x = (screen_width() - width) * 0.5;
    let y = (screen_height() - height) * 0.5;

    ui::panel(
        x,
        y,
        width,
        height,
        rgba(12, 14, 32, 242),
        rgba(94, 242, 255, 72),
    );

    ui::center_text(
        i18n.t(Text::SavesTitle),
        x + width * 0.5,
        y + 50.0 * ui,
        (32.0 * ui).max(22.0),
        GOLD,
    );
    ui::center_text(
        i18n.t(Text::ChooseSlot),
        x + width * 0.5,
        y + 78.0 * ui,
        (14.0 * ui).max(11.0),
        rgba(206, 212, 236, 230),
    );

    let card_x = x + 24.0 * ui;
    let card_w = width - 48.0 * ui;
    let card_h = ((height - 214.0 * ui) / save::SLOT_COUNT as f32 - 10.0 * ui).max(64.0 * ui);
    let gap = 10.0 * ui;
    let first_y = y + 100.0 * ui;

    for slot in 0..save::SLOT_COUNT {
        let card_y = first_y + slot as f32 * (card_h + gap);
        let card = Rect::new(card_x, card_y, card_w, card_h);
        draw_rectangle(
            card.x,
            card.y,
            card.w,
            card.h,
            if pointer.hovering(card) {
                rgba(26, 28, 52, 240)
            } else {
                rgba(20, 22, 42, 230)
            },
        );
        draw_rectangle_lines(card.x, card.y, card.w, card.h, 1.0, rgba(94, 242, 255, 45));

        let title = format!("{} {}", i18n.t(Text::SlotLabel), slot + 1);
        ui::shadow_text(
            &title,
            card.x + 18.0 * ui,
            card.y + 28.0 * ui,
            (19.0 * ui).max(14.0),
            GOLD,
        );

        match menu.slots[slot] {
            None => {
                ui::shadow_text(
                    i18n.t(Text::EmptySlot),
                    card.x + 18.0 * ui,
                    card.y + 54.0 * ui,
                    (13.0 * ui).max(10.0),
                    rgba(148, 158, 190, 230),
                );

                let button = Rect::new(
                    card.x + card.w - 156.0 * ui,
                    card.y + (card.h - 40.0 * ui) * 0.5,
                    136.0 * ui,
                    40.0 * ui,
                );
                if ui::button(pointer, button, i18n.t(Text::NewGame), ui, true, ui::AURA) {
                    return Some(MenuEvent::Start { slot, save: None });
                }
            }
            Some(data) => {
                ui::shadow_text(
                    &slot_summary(i18n, &data),
                    card.x + 18.0 * ui,
                    card.y + 54.0 * ui,
                    (13.0 * ui).max(10.0),
                    rgba(206, 212, 236, 235),
                );

                let load = Rect::new(
                    card.x + card.w - 244.0 * ui,
                    card.y + (card.h - 40.0 * ui) * 0.5,
                    134.0 * ui,
                    40.0 * ui,
                );
                let delete = Rect::new(
                    card.x + card.w - 100.0 * ui,
                    card.y + (card.h - 40.0 * ui) * 0.5,
                    82.0 * ui,
                    40.0 * ui,
                );

                if ui::button(pointer, load, i18n.t(Text::Load), ui, true, ui::AURA) {
                    return Some(MenuEvent::Start {
                        slot,
                        save: Some(data),
                    });
                }
                if ui::button(pointer, delete, i18n.t(Text::Delete), ui, true, ui::RED) {
                    return Some(MenuEvent::RequestDelete(slot));
                }
            }
        }
    }

    let back = Rect::new(x + 24.0 * ui, y + height - 58.0 * ui, 150.0 * ui, 40.0 * ui);
    if ui::button(pointer, back, i18n.t(Text::Back), ui, true, ui::STEEL) {
        return Some(MenuEvent::Back);
    }

    None
}
