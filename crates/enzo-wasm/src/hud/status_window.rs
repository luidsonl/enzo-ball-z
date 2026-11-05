//! Conteúdo da janela de status e da seleção de atributos no level-up.

use enzo_core::input::Attribute;
use enzo_core::snapshot::PlayerSnapshot;
use macroquad::prelude::*;

use crate::i18n::{I18n, Text};
use crate::pointer::Pointer;
use crate::ui;

use super::window::WindowState;
use super::HudAction;

/// Desenha o conteúdo. Retorna uma ação apenas quando o ponteiro clica em um
/// atributo ou no botão de voltar, e apenas se a janela for a mais acima.
pub(super) fn draw(
    pointer: Pointer,
    state: WindowState,
    player: &PlayerSnapshot,
    i18n: I18n,
    ui: f32,
    interactive: bool,
) -> Option<HudAction> {
    let content = state.content_rect(ui);
    let mut action = None;

    let mut y = content.y + 2.0 * ui;
    if player.pending_level_up {
        ui::shadow_text(
            i18n.t(Text::LevelUpTitle),
            content.x,
            y + 18.0 * ui,
            (18.0 * ui).max(13.0),
            ui::GOLD,
        );
        y += 26.0 * ui;
    }

    let summary = format!(
        "{} {}   ·   {:.0}/{:.0} XP",
        i18n.t(Text::LevelShort),
        player.level,
        player.xp,
        player.xp_to_next
    );
    ui::shadow_text(
        &summary,
        content.x,
        y + 14.0 * ui,
        (13.0 * ui).max(10.0),
        ui::rgba(206, 212, 236, 255),
    );
    y += 22.0 * ui;

    let can_spend = player.points > 0;
    if can_spend {
        let points = format!("{} {}", player.points, i18n.t(Text::PointsToSpend));
        ui::center_text(
            &points,
            content.x + content.w * 0.5,
            y + 13.0 * ui,
            (14.0 * ui).max(10.0),
            ui::rgba(255, 227, 138, 255),
        );
        ui::center_text(
            i18n.t(Text::SpendHint),
            content.x + content.w * 0.5,
            y + 30.0 * ui,
            (11.0 * ui).max(8.0),
            ui::rgba(168, 178, 214, 220),
        );
        y += 38.0 * ui;
    }

    let row_h = 40.0 * ui;
    let gap = 5.0 * ui;
    for attribute in Attribute::ALL {
        let rect = Rect::new(content.x, y, content.w, row_h);
        let hovered = interactive && can_spend && pointer.hovering(rect);
        let fill = if hovered {
            ui::rgba(42, 42, 74, 245)
        } else if can_spend {
            ui::rgba(26, 28, 48, 235)
        } else {
            ui::rgba(20, 22, 40, 220)
        };
        draw_rectangle(rect.x, rect.y, rect.w, rect.h, fill);
        draw_rectangle_lines(
            rect.x,
            rect.y,
            rect.w,
            rect.h,
            1.0,
            if hovered {
                ui::GOLD
            } else {
                ui::rgba(94, 242, 255, 45)
            },
        );

        ui::shadow_text(
            i18n.attribute(attribute),
            rect.x + 12.0 * ui,
            rect.y + 17.0 * ui,
            (14.0 * ui).max(10.0),
            WHITE,
        );
        ui::shadow_text(
            effect_text(i18n, attribute),
            rect.x + 12.0 * ui,
            rect.y + 32.0 * ui,
            (10.0 * ui).max(8.0),
            ui::rgba(168, 178, 214, 255),
        );

        let current = format!(
            "{}: {}",
            i18n.t(Text::CurrentValue),
            player.attributes.get(attribute)
        );
        let dims = measure_text(&current, None, (12.0 * ui).max(9.0) as u16, 1.0);
        ui::shadow_text(
            &current,
            rect.x + rect.w - 12.0 * ui - dims.width,
            rect.y + 26.0 * ui,
            (12.0 * ui).max(9.0),
            ui::AURA,
        );

        if interactive && can_spend && pointer.clicked(rect) {
            action = Some(HudAction::Allocate(attribute));
        }

        y += row_h + gap;
    }

    action
}

fn effect_text(i18n: I18n, attribute: Attribute) -> &'static str {
    match attribute {
        Attribute::Speed => i18n.t(Text::EffectSpeed),
        Attribute::Strength => i18n.t(Text::EffectStrength),
        Attribute::Health => i18n.t(Text::EffectHealth),
        Attribute::Aura => i18n.t(Text::EffectAura),
    }
}
