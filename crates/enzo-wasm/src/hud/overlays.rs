//! Banners, dicas e telas de pausa/fim de fase.

use enzo_core::snapshot::WorldSnapshot;
use enzo_core::state::GamePhase;
use macroquad::prelude::*;

use super::find_player;
use super::widgets::{center_text, panel, rgba};

pub(super) fn draw_phase_banner(snapshot: &WorldSnapshot, ui: f32) {
    if snapshot.phase != GamePhase::Playing {
        return;
    }
    let text = "FASE 1  ·  CHEGUE À BANDEIRA";
    let dims = measure_text(text, None, (13.0 * ui) as u16, 1.0);
    let x = (screen_width() - dims.width) * 0.5;
    draw_text(
        text,
        x + 1.0,
        30.0 * ui + 1.0,
        13.0 * ui,
        rgba(8, 8, 18, 160),
    );
    draw_text(text, x, 30.0 * ui, 13.0 * ui, rgba(232, 236, 255, 190));
}

pub(super) fn draw_hints(hint_timer: f32, ui: f32) {
    if hint_timer <= 0.0 {
        return;
    }
    let alpha = (hint_timer / 1.0).clamp(0.0, 1.0);
    let width = (screen_width() - 32.0 * ui).min(780.0 * ui);
    let height = 58.0 * ui;
    let x = (screen_width() - width) * 0.5;
    let y = screen_height() - height - 16.0 * ui;

    draw_rectangle(x, y, width, height, rgba(10, 10, 24, (185.0 * alpha) as u8));
    draw_rectangle_lines(
        x,
        y,
        width,
        height,
        1.0,
        rgba(94, 242, 255, (70.0 * alpha) as u8),
    );

    let line1 = "A/D ou ←/→ mover   ·   ESPAÇO pular   ·   SHIFT dash   ·   J atacar   ·   K Mogar";
    let line2 = "1-4 distribuir pontos   ·   ENTER confirmar   ·   R reiniciar";
    center_text(
        line1,
        x + width * 0.5,
        y + 23.0 * ui,
        (13.0 * ui).max(10.0),
        rgba(232, 236, 255, (235.0 * alpha) as u8),
    );
    center_text(
        line2,
        x + width * 0.5,
        y + 43.0 * ui,
        (12.0 * ui).max(9.0),
        rgba(168, 178, 214, (220.0 * alpha) as u8),
    );
}

pub(super) fn draw_save_toast(save_toast: Option<(&str, f32)>, ui: f32) {
    let Some((text, remaining)) = save_toast else {
        return;
    };
    if remaining <= 0.0 {
        return;
    }
    let alpha = (remaining / 0.6).clamp(0.0, 1.0);
    let label = format!("SALVO · {text}");
    let dims = measure_text(&label, None, (13.0 * ui) as u16, 1.0);
    let w = dims.width + 24.0 * ui;
    let h = 28.0 * ui;
    let x = screen_width() - w - 16.0 * ui;
    let y = 16.0 * ui;
    draw_rectangle(x, y, w, h, rgba(12, 12, 26, (210.0 * alpha) as u8));
    draw_rectangle_lines(x, y, w, h, 1.0, rgba(246, 196, 69, (130.0 * alpha) as u8));
    draw_text(
        &label,
        x + 12.0 * ui,
        y + 18.0 * ui,
        13.0 * ui,
        rgba(255, 227, 138, (255.0 * alpha) as u8),
    );
}

pub(super) fn draw_level_up(snapshot: &WorldSnapshot, ui: f32) {
    let Some((_, player)) = find_player(snapshot) else {
        return;
    };
    let width = (screen_width() - 40.0 * ui).min(640.0 * ui);
    let height = (screen_height() - 40.0 * ui).min(380.0 * ui);
    let x = (screen_width() - width) * 0.5;
    let y = (screen_height() - height) * 0.5;

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        rgba(6, 6, 16, 200),
    );
    panel(
        x,
        y,
        width,
        height,
        rgba(16, 16, 34, 245),
        rgba(246, 196, 69, 180),
    );

    center_text(
        "SUBIU DE NÍVEL!",
        x + width * 0.5,
        y + 52.0 * ui,
        (32.0 * ui).max(22.0),
        rgba(246, 196, 69, 255),
    );
    center_text(
        &format!("Nível {}", player.level),
        x + width * 0.5,
        y + 82.0 * ui,
        (18.0 * ui).max(14.0),
        WHITE,
    );
    center_text(
        &format!("{} ponto(s) para distribuir", player.points),
        x + width * 0.5,
        y + 106.0 * ui,
        (14.0 * ui).max(11.0),
        rgba(206, 212, 236, 255),
    );

    let row_h = 44.0 * ui;
    let row_w = width - 48.0 * ui;
    let row_x = x + 24.0 * ui;
    let first_y = y + 128.0 * ui;

    let rows = [
        ("1", "Velocidade", "+10 px/s", player.attributes.speed),
        ("2", "Força", "+2 dano físico", player.attributes.strength),
        ("3", "Vida", "+10 vida máxima", player.attributes.health),
        ("4", "Aura", "+10 aura / +2 Mogar", player.attributes.aura),
    ];

    for (index, (key, name, effect, current)) in rows.iter().enumerate() {
        let ry = first_y + index as f32 * (row_h + 6.0 * ui);
        draw_rectangle(
            row_x,
            ry,
            row_w,
            row_h,
            if player.points > 0 {
                rgba(28, 28, 52, 235)
            } else {
                rgba(22, 22, 40, 210)
            },
        );
        draw_rectangle_lines(row_x, ry, row_w, row_h, 1.0, rgba(94, 242, 255, 45));

        draw_rectangle(
            row_x + 8.0 * ui,
            ry + 8.0 * ui,
            28.0 * ui,
            28.0 * ui,
            rgba(246, 196, 69, 220),
        );
        center_text(
            key,
            row_x + 22.0 * ui,
            ry + 28.0 * ui,
            (18.0 * ui).max(13.0),
            rgba(20, 16, 10, 255),
        );

        draw_text(
            name,
            row_x + 48.0 * ui,
            ry + 20.0 * ui,
            (16.0 * ui).max(12.0),
            WHITE,
        );
        draw_text(
            effect,
            row_x + 48.0 * ui,
            ry + 36.0 * ui,
            (11.0 * ui).max(9.0),
            rgba(168, 178, 214, 255),
        );

        let current_text = format!("atual: {current}");
        let dims = measure_text(&current_text, None, (13.0 * ui) as u16, 1.0);
        draw_text(
            &current_text,
            row_x + row_w - 12.0 * ui - dims.width,
            ry + 27.0 * ui,
            13.0 * ui,
            rgba(94, 242, 255, 255),
        );
    }

    let footer = if player.points > 0 {
        "Pressione 1-4 para gastar e ENTER para voltar"
    } else {
        "ENTER para voltar ao jogo"
    };
    center_text(
        footer,
        x + width * 0.5,
        y + height - 20.0 * ui,
        (14.0 * ui).max(11.0),
        if player.points > 0 {
            rgba(206, 212, 236, 255)
        } else {
            rgba(143, 224, 122, 255)
        },
    );
}

pub(super) fn draw_dead(snapshot: &WorldSnapshot, ui: f32) {
    let width = (screen_width() - 40.0 * ui).min(560.0 * ui);
    let height = 220.0 * ui;
    let x = (screen_width() - width) * 0.5;
    let y = (screen_height() - height) * 0.5;

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        rgba(30, 4, 8, 190),
    );
    panel(
        x,
        y,
        width,
        height,
        rgba(24, 10, 18, 240),
        rgba(232, 72, 63, 180),
    );

    center_text(
        "VOCÊ CAIU!",
        x + width * 0.5,
        y + 58.0 * ui,
        (34.0 * ui).max(24.0),
        rgba(255, 128, 116, 255),
    );
    center_text(
        &format!("Renascendo em {:.1}s…", snapshot.phase_timer.max(0.0)),
        x + width * 0.5,
        y + 104.0 * ui,
        (18.0 * ui).max(14.0),
        WHITE,
    );
    center_text(
        "A progressão foi mantida.",
        x + width * 0.5,
        y + 138.0 * ui,
        (14.0 * ui).max(11.0),
        rgba(206, 212, 236, 255),
    );
    center_text(
        "R para reiniciar agora",
        x + width * 0.5,
        y + 180.0 * ui,
        (13.0 * ui).max(10.0),
        rgba(148, 158, 190, 255),
    );
}

pub(super) fn draw_victory(snapshot: &WorldSnapshot, ui: f32) {
    let Some((_, player)) = find_player(snapshot) else {
        return;
    };
    let width = (screen_width() - 40.0 * ui).min(620.0 * ui);
    let height = 300.0 * ui;
    let x = (screen_width() - width) * 0.5;
    let y = (screen_height() - height) * 0.5;

    draw_rectangle(
        0.0,
        0.0,
        screen_width(),
        screen_height(),
        rgba(6, 10, 24, 205),
    );
    panel(
        x,
        y,
        width,
        height,
        rgba(14, 18, 38, 245),
        rgba(246, 196, 69, 200),
    );

    center_text(
        "FASE CONCLUÍDA!",
        x + width * 0.5,
        y + 60.0 * ui,
        (34.0 * ui).max(24.0),
        rgba(246, 196, 69, 255),
    );
    center_text(
        &format!(
            "Nível {}  ·  {:.0}/{:.0} XP",
            player.level, player.xp, player.xp_to_next
        ),
        x + width * 0.5,
        y + 104.0 * ui,
        (18.0 * ui).max(14.0),
        WHITE,
    );
    center_text(
        &format!(
            "VEL {}   FOR {}   VIT {}   AUR {}",
            player.attributes.speed,
            player.attributes.strength,
            player.attributes.health,
            player.attributes.aura,
        ),
        x + width * 0.5,
        y + 138.0 * ui,
        (15.0 * ui).max(11.0),
        rgba(206, 212, 236, 255),
    );
    center_text(
        "Progresso salvo no navegador.",
        x + width * 0.5,
        y + 184.0 * ui,
        (14.0 * ui).max(11.0),
        rgba(143, 224, 122, 255),
    );
    center_text(
        "Pressione R para jogar de novo",
        x + width * 0.5,
        y + 236.0 * ui,
        (16.0 * ui).max(12.0),
        rgba(94, 242, 255, 255),
    );
}
