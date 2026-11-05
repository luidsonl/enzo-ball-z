//! HUD e overlays em screen space.
//!
//! Tudo aqui é leitura do snapshot: barras, nível, pontos, dicas e as
//! telas de level-up, morte e vitória. Nenhuma regra de jogo é decidida aqui.

use enzo_core::entities::EquipmentTags;
use enzo_core::input::Attribute;
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot, SnapshotKind, WorldSnapshot};
use enzo_core::world::GamePhase;
use macroquad::prelude::*;

use crate::effects::Effects;

pub fn draw(
    snapshot: &WorldSnapshot,
    effects: &Effects,
    time: f32,
    hint_timer: f32,
    save_toast: Option<(&str, f32)>,
) {
    set_default_camera();
    let ui = (screen_height() / 720.0).clamp(0.8, 2.0);

    if let Some((entity, player)) = find_player(snapshot) {
        draw_status_panel(entity, player, effects, time, ui);
    }
    draw_phase_banner(snapshot, ui);
    draw_hints(hint_timer, ui);
    draw_save_toast(save_toast, ui);

    match snapshot.phase {
        GamePhase::Playing => {}
        GamePhase::LevelUp => draw_level_up(snapshot, ui),
        GamePhase::Dead => draw_dead(snapshot, ui),
        GamePhase::Victory => draw_victory(snapshot, ui),
    }

    // Flashes por cima da UI, mas abaixo dos overlays de pausa.
    if effects.hurt_flash > 0.0 && snapshot.phase != GamePhase::Dead {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            rgba(255, 60, 50, (effects.hurt_flash * 70.0) as u8),
        );
    }
    if effects.victory_flash > 0.0 {
        draw_rectangle(
            0.0,
            0.0,
            screen_width(),
            screen_height(),
            rgba(246, 196, 69, (effects.victory_flash * 40.0) as u8),
        );
    }
}

pub fn attribute_name(attribute: Attribute) -> &'static str {
    match attribute {
        Attribute::Speed => "Velocidade",
        Attribute::Strength => "Força",
        Attribute::Health => "Vida",
        Attribute::Aura => "Aura",
    }
}

fn find_player(snapshot: &WorldSnapshot) -> Option<(&EntitySnapshot, &PlayerSnapshot)> {
    snapshot
        .entities
        .iter()
        .find_map(|entity| match &entity.kind {
            SnapshotKind::Player(player) => Some((entity, player)),
            _ => None,
        })
}

fn draw_status_panel(
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

fn draw_phase_banner(snapshot: &WorldSnapshot, ui: f32) {
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

fn draw_hints(hint_timer: f32, ui: f32) {
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

fn draw_save_toast(save_toast: Option<(&str, f32)>, ui: f32) {
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

fn draw_level_up(snapshot: &WorldSnapshot, ui: f32) {
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

fn draw_dead(snapshot: &WorldSnapshot, ui: f32) {
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

fn draw_victory(snapshot: &WorldSnapshot, ui: f32) {
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

fn panel(x: f32, y: f32, w: f32, h: f32, fill: Color, border: Color) {
    draw_rectangle(x, y, w, h, fill);
    draw_rectangle_lines(x, y, w, h, 2.0, border);
    draw_rectangle_lines(
        x + 3.0,
        y + 3.0,
        w - 6.0,
        h - 6.0,
        1.0,
        rgba(255, 255, 255, 20),
    );
}

fn center_text(text: &str, cx: f32, y: f32, size: f32, color: Color) {
    let dims = measure_text(text, None, size as u16, 1.0);
    draw_text(text, cx - dims.width * 0.5, y, size, color);
}

fn rgba(r: u8, g: u8, b: u8, a: u8) -> Color {
    Color::from_rgba(r, g, b, a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attribute_labels_are_in_portuguese() {
        assert_eq!(attribute_name(Attribute::Speed), "Velocidade");
        assert_eq!(attribute_name(Attribute::Strength), "Força");
        assert_eq!(attribute_name(Attribute::Health), "Vida");
        assert_eq!(attribute_name(Attribute::Aura), "Aura");
    }
}
