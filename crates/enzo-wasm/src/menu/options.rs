//! Tela de opções do menu. Por enquanto, idioma.

use macroquad::prelude::*;

use super::MenuEvent;
use crate::i18n::{I18n, Text, LANGUAGES};
use crate::pointer::Pointer;
use crate::ui::{self, rgba, GOLD};

pub(super) fn draw(pointer: Pointer, i18n: I18n, ui: f32, _time: f32) -> Option<MenuEvent> {
    let width = (screen_width() * 0.62).clamp(380.0 * ui, 560.0 * ui);
    let height = (screen_height() * 0.52).clamp(300.0 * ui, 380.0 * ui);
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
        i18n.t(Text::OptionsTitle),
        x + width * 0.5,
        y + 50.0 * ui,
        (32.0 * ui).max(22.0),
        GOLD,
    );
    ui::center_text(
        i18n.t(Text::Language),
        x + width * 0.5,
        y + 92.0 * ui,
        (15.0 * ui).max(11.0),
        rgba(206, 212, 236, 230),
    );

    let button_w = width - 72.0 * ui;
    let button_h = 46.0 * ui;
    let button_x = x + 36.0 * ui;
    let mut event = None;

    for (index, language) in LANGUAGES.iter().enumerate() {
        let rect = Rect::new(
            button_x,
            y + 112.0 * ui + index as f32 * (button_h + 12.0 * ui),
            button_w,
            button_h,
        );
        let selected = i18n.language() == *language;
        let accent = if selected { GOLD } else { ui::AURA };
        let label = if selected {
            format!("▶  {}", language.label())
        } else {
            language.label().to_owned()
        };
        if ui::button(pointer, rect, &label, ui, true, accent) && !selected {
            event = Some(MenuEvent::SetLanguage(*language));
        }
    }

    let back = Rect::new(x + 24.0 * ui, y + height - 58.0 * ui, 150.0 * ui, 40.0 * ui);
    if ui::button(pointer, back, i18n.t(Text::Back), ui, true, ui::STEEL) {
        return Some(MenuEvent::Back);
    }

    event
}
