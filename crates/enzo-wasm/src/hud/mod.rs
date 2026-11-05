//! HUD e overlays em screen space.
//!
//! Tudo aqui é leitura do snapshot: barras, nível, pontos, dicas e as telas de
//! level-up, morte e vitória. Nenhuma regra de jogo é decidida aqui.
//!
//! O desenho é dividido por responsabilidade:
//! - [`status`] desenha o painel persistente do jogador;
//! - [`overlays`] desenha banners, dicas e telas de pausa/fim;
//! - [`widgets`] concentra primitivas visuais compartilhadas.

mod overlays;
mod status;
mod widgets;

use enzo_core::input::Attribute;
use enzo_core::snapshot::{EntitySnapshot, PlayerSnapshot, SnapshotKind, WorldSnapshot};
use enzo_core::state::GamePhase;
use macroquad::prelude::*;

use crate::effects::Effects;

use overlays::{
    draw_dead, draw_hints, draw_level_up, draw_phase_banner, draw_save_toast, draw_victory,
};
use status::draw_status_panel;
use widgets::rgba;

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

pub(super) fn find_player(snapshot: &WorldSnapshot) -> Option<(&EntitySnapshot, &PlayerSnapshot)> {
    snapshot
        .entities
        .iter()
        .find_map(|entity| match &entity.kind {
            SnapshotKind::Player(player) => Some((entity, player)),
            _ => None,
        })
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
