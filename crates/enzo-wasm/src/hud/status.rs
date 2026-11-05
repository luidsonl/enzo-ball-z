//! Painel persistente do jogador: vida, aura, XP, atributos e Mogar.

use enzo_core::entities::EquipmentTags;
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot};
use macroquad::prelude::*;

use crate::effects::Effects;

use super::widgets::{panel, rgba};

pub(super) fn draw_status_panel(
    entity: &EntitySnapshot,
    player: &PlayerSnapshot,
    effects: &Effects,
    time: f32,
    ui: f32,
) {
    let x = 16.0 * ui;
    let y = 16.0 * ui;
    let w = 370.0 * ui;
    let h = 166.0 * ui;
    panel(x, y, w, h, rgba(12, 12, 26, 218), rgba(94, 242, 255, 70));

    let pad = 14.0 * ui;
    draw_text("ENZO", x + pad, y + 28.0 * ui, 24.0 * ui, WHITE);
    let level = format!("Nv. {}", player.level);
    draw_text(
        &level,
        x + pad + 76.0 * ui,
        y + 28.0 * ui,
        20.0 * ui,
        rgba(246, 196, 69, 255),
    );

    if player.points > 0 {
        let points = format!("{} PONTO(S)", player.points);
        let dims = measure_text(&points, None, (13.0 * ui) as u16, 1.0);
        let px = x + w - pad - dims.width;
        let pulse = 0.5 + 0.5 * (time * 5.0).sin();
        draw_text(
            &points,
            px,
            y + 27.0 * ui,
            13.0 * ui,
            rgba(255, 227, 138, (170.0 + 85.0 * pulse) as u8),
        );
    }

    let bar_x = x + pad;
    let bar_w = w - pad * 2.0;
    let hp_ratio = (entity.health / entity.max_health).clamp(0.0, 1.0);
    let aura_ratio = (entity.aura / entity.max_aura.max(1.0)).clamp(0.0, 1.0);
    let xp_ratio = (player.xp / player.xp_to_next.max(1.0)).clamp(0.0, 1.0);

    draw_bar(
        bar_x,
        y + 42.0 * ui,
        bar_w,
        16.0 * ui,
        hp_ratio,
        rgba(232, 72, 63, 255),
        rgba(255, 128, 116, 255),
    );
    bar_label(
        bar_x,
        y + 42.0 * ui,
        bar_w,
        16.0 * ui,
        &format!("VIDA  {:.0}/{:.0}", entity.health, entity.max_health),
        ui,
    );

    let aura_flash = effects.aura_flash > 0.0;
    draw_bar(
        bar_x,
        y + 64.0 * ui,
        bar_w,
        16.0 * ui,
        aura_ratio,
        if aura_flash {
            rgba(220, 255, 255, 255)
        } else {
            rgba(58, 196, 220, 255)
        },
        rgba(94, 242, 255, 255),
    );
    bar_label(
        bar_x,
        y + 64.0 * ui,
        bar_w,
        16.0 * ui,
        &format!("AURA  {:.0}/{:.0}", entity.aura, entity.max_aura),
        ui,
    );

    draw_bar(
        bar_x,
        y + 86.0 * ui,
        bar_w,
        12.0 * ui,
        xp_ratio,
        rgba(196, 148, 34, 255),
        rgba(246, 196, 69, 255),
    );
    bar_label(
        bar_x,
        y + 86.0 * ui,
        bar_w,
        12.0 * ui,
        &format!("XP  {:.0}/{:.0}", player.xp, player.xp_to_next),
        ui,
    );

    // Atributos investidos.
    let attrs = format!(
        "VEL {}   FOR {}   VIT {}   AUR {}",
        player.attributes.speed,
        player.attributes.strength,
        player.attributes.health,
        player.attributes.aura,
    );
    draw_text(
        &attrs,
        x + pad,
        y + 118.0 * ui,
        14.0 * ui,
        rgba(206, 212, 236, 255),
    );

    // Mogar e equipamento.
    let mogar = if player.mogar_cooldown <= 0.0 {
        "MOGAR PRONTO (25 aura)".to_string()
    } else {
        format!("MOGAR {:.2}s", player.mogar_cooldown)
    };
    draw_text(
        &mogar,
        x + pad,
        y + 142.0 * ui,
        12.0 * ui,
        if player.mogar_cooldown <= 0.0 {
            rgba(176, 150, 255, 255)
        } else {
            rgba(120, 112, 150, 255)
        },
    );

    if let Some(equipment) = equipment_label(player) {
        let dims = measure_text(&equipment, None, (12.0 * ui) as u16, 1.0);
        draw_text(
            &equipment,
            x + w - pad - dims.width,
            y + 142.0 * ui,
            12.0 * ui,
            rgba(246, 196, 69, 210),
        );
    }
}

fn equipment_label(player: &PlayerSnapshot) -> Option<String> {
    let equipment = &player.equipment;
    if equipment.tags.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    if equipment.has(EquipmentTags::SWORD) {
        parts.push("ESPADA");
    }
    if equipment.has(EquipmentTags::BOW) {
        parts.push("ARCO");
    }
    if parts.is_empty() {
        None
    } else {
        Some(format!("EQUIP: {}", parts.join(" + ")))
    }
}

fn draw_bar(x: f32, y: f32, w: f32, h: f32, ratio: f32, fill: Color, light: Color) {
    draw_rectangle(x - 1.0, y - 1.0, w + 2.0, h + 2.0, rgba(4, 4, 12, 220));
    draw_rectangle(x, y, w, h, rgba(38, 38, 58, 230));
    let filled = (w * ratio.clamp(0.0, 1.0)).max(0.0);
    if filled > 0.0 {
        draw_rectangle(x, y, filled, h, fill);
        draw_rectangle(x, y, filled, (h * 0.35).max(1.0), light);
    }
    draw_rectangle_lines(x, y, w, h, 1.0, rgba(150, 160, 200, 90));
}

fn bar_label(x: f32, y: f32, w: f32, h: f32, text: &str, ui: f32) {
    let size = (11.0 * ui).max(9.0);
    let dims = measure_text(text, None, size as u16, 1.0);
    let tx = x + (w - dims.width) * 0.5;
    let ty = y + h * 0.5 + size * 0.36;
    draw_text(text, tx + 1.0, ty + 1.0, size, rgba(4, 4, 12, 220));
    draw_text(text, tx, ty, size, WHITE);
}
