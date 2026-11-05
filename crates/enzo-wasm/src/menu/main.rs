//! Tela principal do menu.

use macroquad::prelude::*;

use super::{MenuEvent, MenuScreen};
use crate::i18n::{I18n, Text};
use crate::pointer::Pointer;
use crate::ui::{self, rgba, GOLD};

pub(super) fn draw(pointer: Pointer, i18n: I18n, ui: f32, time: f32) -> Option<MenuEvent> {
    let width = (screen_width() * 0.62).clamp(380.0 * ui, 560.0 * ui);
    let height = (screen_height() * 0.58).clamp(330.0 * ui, 430.0 * ui);
    let x = (screen_width() - width) * 0.5;
    let y = (screen_height() - height) * 0.5;

    // Título com brilho pulsante, fora do painel para dar hierarquia.
    let pulse = 0.5 + 0.5 * (time * 1.6).sin();
    ui::center_text(
        i18n.t(Text::MainTitle),
        screen_width() * 0.5,
        y - 28.0 * ui,
        (54.0 * ui).max(32.0),
        rgba(246, 196, 69, (185.0 + 70.0 * pulse) as u8),
    );
    ui::center_text(
        i18n.t(Text::MenuSubtitle),
        screen_width() * 0.5,
        y + 4.0 * ui,
        (17.0 * ui).max(12.0),
        rgba(206, 212, 236, 235),
    );

    ui::panel(
        x,
        y + 24.0 * ui,
        width,
        height,
        rgba(12, 14, 32, 238),
        rgba(94, 242, 255, 72),
    );

    let button_w = width - 72.0 * ui;
    let button_h = 56.0 * ui;
    let button_x = x + 36.0 * ui;
    let play = Rect::new(button_x, y + 84.0 * ui, button_w, button_h);
    let options = Rect::new(button_x, play.y + button_h + 22.0 * ui, button_w, button_h);

    if ui::button(pointer, play, i18n.t(Text::Play), ui, true, ui::AURA) {
        return Some(MenuEvent::Push(MenuScreen::Saves));
    }
    if ui::button(
        pointer,
        options,
        i18n.t(Text::Options),
        ui,
        true,
        ui::PURPLE,
    ) {
        return Some(MenuEvent::Push(MenuScreen::Options));
    }

    ui::center_text(
        i18n.t(Text::MenuHint),
        x + width * 0.5,
        y + height - 26.0 * ui,
        (13.0 * ui).max(10.0),
        rgba(148, 158, 190, 220),
    );

    // Detalhe visual: três pontos coloridos representando os atributos.
    let dot_y = y + height + 42.0 * ui;
    for (index, color) in [ui::AURA, GOLD, ui::RED].iter().enumerate() {
        let dot_x = screen_width() * 0.5 + (index as f32 - 1.0) * 26.0 * ui;
        let radius = 4.0 * ui + pulse * 1.5;
        draw_circle(dot_x, dot_y, radius, *color);
    }

    None
}
