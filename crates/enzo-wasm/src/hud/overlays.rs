//! Banners, dicas e telas de morte/vitória.
//!
//! As telas de fim têm botões clicáveis; o teclado (R/Enter) continua valendo
//! porque o `app` ainda envia os mesmos comandos ao core.

use enzo_core::snapshot::WorldSnapshot;
use enzo_core::state::GamePhase;
use macroquad::prelude::*;

use crate::i18n::{I18n, Text};
use crate::pointer::Pointer;
use crate::ui;

use super::find_player;
use super::HudAction;

pub(super) fn draw_phase_banner(snapshot: &WorldSnapshot, i18n: I18n, ui: f32) {
    if snapshot.phase != GamePhase::Playing {
        return;
    }
    let text = i18n.t(Text::PhaseBanner);
    let dims = measure_text(text, None, (13.0 * ui) as u16, 1.0);
    let x = (screen_width() - dims.width) * 0.5;
    draw_text(
        text,
        x + 1.0,
        30.0 * ui + 1.0,
        13.0 * ui,
        ui::rgba(8, 8, 18, 160),
    );
    draw_text(text, x, 30.0 * ui, 13.0 * ui, ui::rgba(232, 236, 255, 190));
}

pub(super) fn draw_hints(hint_timer: f32, i18n: I18n, ui: f32) {
    if hint_timer <= 0.0 {
        return;
    }
    let alpha = (hint_timer / 1.0).clamp(0.0, 1.0);
    let width = (screen_width() - 32.0 * ui).min(820.0 * ui);
    let height = 58.0 * ui;
    let x = (screen_width() - width) * 0.5;
    let y = screen_height() - height - 16.0 * ui;

    draw_rectangle(
        x,
        y,
        width,
        height,
        ui::rgba(10, 10, 24, (185.0 * alpha) as u8),
    );
    draw_rectangle_lines(
        x,
        y,
        width,
        height,
        1.0,
        ui::rgba(94, 242, 255, (70.0 * alpha) as u8),
    );

    ui::center_text(
        i18n.t(Text::ControlsLine1),
        x + width * 0.5,
        y + 23.0 * ui,
        (13.0 * ui).max(10.0),
        ui::rgba(232, 236, 255, (235.0 * alpha) as u8),
    );
    ui::center_text(
        i18n.t(Text::ControlsLine2),
        x + width * 0.5,
        y + 43.0 * ui,
        (12.0 * ui).max(9.0),
        ui::rgba(168, 178, 214, (220.0 * alpha) as u8),
    );
}

pub(super) fn draw_save_toast(save_toast: Option<(&str, f32)>, i18n: I18n, ui: f32) {
    let Some((text, remaining)) = save_toast else {
        return;
    };
    if remaining <= 0.0 {
        return;
    }
    let alpha = (remaining / 0.6).clamp(0.0, 1.0);
    let label = format!("{} · {text}", i18n.t(Text::SavedPrefix));
    let size = (13.0 * ui).max(9.0);
    let dims = measure_text(&label, None, size as u16, 1.0);
    let w = dims.width + 24.0 * ui;
    let h = 28.0 * ui;
    let x = screen_width() - w - 16.0 * ui;
    let y = 54.0 * ui;
    draw_rectangle(x, y, w, h, ui::rgba(12, 12, 26, (210.0 * alpha) as u8));
    draw_rectangle_lines(
        x,
        y,
        w,
        h,
        1.0,
        ui::rgba(246, 196, 69, (130.0 * alpha) as u8),
    );
    draw_text(
        &label,
        x + 12.0 * ui,
        y + 18.0 * ui,
        size,
        ui::rgba(255, 227, 138, (255.0 * alpha) as u8),
    );
}

pub(super) fn draw_level_up_banner(
    player: &enzo_core::snapshot::PlayerSnapshot,
    i18n: I18n,
    ui: f32,
) {
    let width = (screen_width() - 40.0 * ui).min(420.0 * ui);
    let h = 46.0 * ui;
    let x = (screen_width() - width) * 0.5;
    let y = 16.0 * ui;

    ui::panel(
        x,
        y,
        width,
        h,
        ui::rgba(16, 16, 34, 235),
        ui::rgba(246, 196, 69, 170),
    );
    ui::center_text(
        i18n.t(Text::LevelUpTitle),
        x + width * 0.5,
        y + 21.0 * ui,
        (18.0 * ui).max(13.0),
        ui::GOLD,
    );
    if player.points > 0 {
        ui::center_text(
            &format!("{} {}", player.points, i18n.t(Text::PointsToSpend)),
            x + width * 0.5,
            y + 37.0 * ui,
            (11.0 * ui).max(8.0),
            ui::rgba(255, 227, 138, 240),
        );
    }
}

pub(super) fn draw_dead(
    snapshot: &WorldSnapshot,
    i18n: I18n,
    pointer: Pointer,
    ui: f32,
) -> Option<HudAction> {
    let width = (screen_width() - 40.0 * ui).min(560.0 * ui);
    let height = 250.0 * ui;
    let x = (screen_width() - width) * 0.5;
    let y = (screen_height() - height) * 0.5;

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        ui::rgba(30, 4, 8, 190),
    );
    ui::panel(
        x,
        y,
        width,
        height,
        ui::rgba(24, 10, 18, 240),
        ui::rgba(232, 72, 63, 180),
    );

    ui::center_text(
        i18n.t(Text::DeathTitle),
        x + width * 0.5,
        y + 54.0 * ui,
        (34.0 * ui).max(24.0),
        ui::rgba(255, 128, 116, 255),
    );
    ui::center_text(
        &format!(
            "{} {:.1}s…",
            i18n.t(Text::RespawnIn),
            snapshot.phase_timer.max(0.0)
        ),
        x + width * 0.5,
        y + 94.0 * ui,
        (18.0 * ui).max(14.0),
        WHITE,
    );
    ui::center_text(
        i18n.t(Text::ProgressionKept),
        x + width * 0.5,
        y + 124.0 * ui,
        (14.0 * ui).max(11.0),
        ui::rgba(206, 212, 236, 255),
    );

    let button_w = (width - 72.0 * ui) * 0.5;
    let button_h = 42.0 * ui;
    let by = y + height - 62.0 * ui;
    let restart = Rect::new(x + 24.0 * ui, by, button_w, button_h);
    let menu = Rect::new(x + 48.0 * ui + button_w, by, button_w, button_h);

    let mut action = None;
    if ui::button(
        pointer,
        restart,
        i18n.t(Text::RestartNow),
        ui,
        true,
        ui::GOLD,
    ) {
        action = Some(HudAction::Restart);
    }
    if ui::button(
        pointer,
        menu,
        i18n.t(Text::ReturnToMenu),
        ui,
        true,
        ui::STEEL,
    ) {
        action = Some(HudAction::OpenMenu);
    }
    action
}

pub(super) fn draw_victory(
    snapshot: &WorldSnapshot,
    i18n: I18n,
    pointer: Pointer,
    ui: f32,
) -> Option<HudAction> {
    let (_, player) = find_player(snapshot)?;
    let width = (screen_width() - 40.0 * ui).min(620.0 * ui);
    let height = 310.0 * ui;
    let x = (screen_width() - width) * 0.5;
    let y = (screen_height() - height) * 0.5;

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        ui::rgba(6, 10, 24, 205),
    );
    ui::panel(
        x,
        y,
        width,
        height,
        ui::rgba(14, 18, 38, 245),
        ui::rgba(246, 196, 69, 200),
    );

    ui::center_text(
        i18n.t(Text::VictoryTitle),
        x + width * 0.5,
        y + 58.0 * ui,
        (34.0 * ui).max(24.0),
        ui::GOLD,
    );
    ui::center_text(
        &format!(
            "{} {}  ·  {:.0}/{:.0} XP",
            i18n.t(Text::LevelLabel),
            player.level,
            player.xp,
            player.xp_to_next
        ),
        x + width * 0.5,
        y + 100.0 * ui,
        (18.0 * ui).max(14.0),
        WHITE,
    );
    ui::center_text(
        &format!(
            "{} {}   ·   {} {}   ·   {} {}   ·   {} {}",
            i18n.t(Text::AttributeSpeed),
            player.attributes.speed,
            i18n.t(Text::AttributeStrength),
            player.attributes.strength,
            i18n.t(Text::AttributeHealth),
            player.attributes.health,
            i18n.t(Text::AttributeAura),
            player.attributes.aura,
        ),
        x + width * 0.5,
        y + 134.0 * ui,
        (13.0 * ui).max(10.0),
        ui::rgba(206, 212, 236, 255),
    );
    ui::center_text(
        i18n.t(Text::ProgressSaved),
        x + width * 0.5,
        y + 172.0 * ui,
        (14.0 * ui).max(11.0),
        ui::GREEN,
    );

    let button_w = (width - 72.0 * ui) * 0.5;
    let button_h = 44.0 * ui;
    let by = y + height - 66.0 * ui;
    let again = Rect::new(x + 24.0 * ui, by, button_w, button_h);
    let menu = Rect::new(x + 48.0 * ui + button_w, by, button_w, button_h);

    let mut action = None;
    if ui::button(pointer, again, i18n.t(Text::PlayAgain), ui, true, ui::GOLD) {
        action = Some(HudAction::Restart);
    }
    if ui::button(
        pointer,
        menu,
        i18n.t(Text::ReturnToMenu),
        ui,
        true,
        ui::STEEL,
    ) {
        action = Some(HudAction::OpenMenu);
    }
    action
}
